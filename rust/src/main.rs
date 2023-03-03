#![no_main]
#![no_std]
#![feature(abi_efiapi)]
#![feature(negative_impls)]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate alloc;

mod linux_loader;
mod pe_loader;
mod pe_section;
mod uefi_helpers;

use alloc::vec::Vec;
use log::info;
use pe_loader::Image;
use pe_section::pe_section_as_string;
use uefi::{
    prelude::*,
    proto::media::file::{File, FileAttribute, FileMode, RegularFile},
    CStr16, CString16, Result,
};

use crate::{
    linux_loader::InitrdLoader,
    uefi_helpers::{booted_image_file, read_all},
};

/// Print the startup logo on boot.
fn print_logo() {
    info!(
        "
  _                      _                 _
 | |                    | |               | |
 | | __ _ _ __  ______ _| |__   ___   ___ | |_ ___
 | |/ _` | '_ \\|_  / _` | '_ \\ / _ \\ / _ \\| __/ _ \\
 | | (_| | | | |/ / (_| | |_) | (_) | (_) | ||  __/
 |_|\\__,_|_| |_/___\\__,_|_.__/ \\___/ \\___/ \\__\\___|

"
    );
}

/// The configuration that is embedded at build time.
///
/// After lanzaboote is built, lzbt needs to embed configuration
/// into the binary. This struct represents that information.
struct EmbeddedConfiguration {
    /// The filename of the kernel to be booted. This filename is
    /// relative to the root of the volume that contains the
    /// lanzaboote binary.
    kernel_filename: CString16,

    /// The filename of the initrd to be passed to the kernel. See
    /// `kernel_filename` for how to interpret these filenames.
    initrd_filename: CString16,

    /// The kernel command-line.
    cmdline: CString16,
}

/// Extract a string, stored as UTF-8, from a PE section.
fn extract_string(file_data: &[u8], section: &str) -> Result<CString16> {
    let string = pe_section_as_string(file_data, section).ok_or(Status::INVALID_PARAMETER)?;

    Ok(CString16::try_from(string.as_str()).map_err(|_| Status::INVALID_PARAMETER)?)
}

impl EmbeddedConfiguration {
    fn new(file: &mut RegularFile) -> Result<Self> {
        file.set_position(0)?;
        let file_data = read_all(file)?;

        Ok(Self {
            kernel_filename: extract_string(&file_data, ".kernelp")?,

            initrd_filename: extract_string(&file_data, ".initrdp")?,

            cmdline: extract_string(&file_data, ".cmdline")?,
        })
    }
}

/// Boot the Linux kernel without checking the PE signature.
///
/// We assume that the caller has made sure that the image is safe to
/// be loaded using other means.
fn boot_linux_unchecked(
    handle: Handle,
    system_table: SystemTable<Boot>,
    kernel_data: Vec<u8>,
    kernel_cmdline: &CStr16,
    initrd_data: Vec<u8>,
) -> uefi::Result<()> {
    let kernel =
        Image::load(system_table.boot_services(), &kernel_data).expect("Failed to load the kernel");

    let mut initrd_loader = InitrdLoader::new(system_table.boot_services(), handle, initrd_data)?;

    let status = unsafe { kernel.start(handle, &system_table, kernel_cmdline) };

    initrd_loader.uninstall(system_table.boot_services())?;
    status.into()
}

#[entry]
fn main(handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap();

    print_logo();

    let config: EmbeddedConfiguration =
        EmbeddedConfiguration::new(&mut booted_image_file(system_table.boot_services()).unwrap())
            .expect("Failed to extract configuration from binary. Did you run lzbt?");

    let kernel_data;
    let initrd_data;

    {
        let mut file_system = system_table
            .boot_services()
            .get_image_file_system(handle)
            .expect("Failed to get file system handle");
        let mut root = file_system
            .open_volume()
            .expect("Failed to find ESP root directory");

        let mut kernel_file = root
            .open(
                &config.kernel_filename,
                FileMode::Read,
                FileAttribute::empty(),
            )
            .expect("Failed to open kernel file for reading")
            .into_regular_file()
            .expect("Kernel is not a regular file");

        kernel_data = read_all(&mut kernel_file).expect("Failed to read kernel file into memory");

        let mut initrd_file = root
            .open(
                &config.initrd_filename,
                FileMode::Read,
                FileAttribute::empty(),
            )
            .expect("Failed to open initrd for reading")
            .into_regular_file()
            .expect("Initrd is not a regular file");

        initrd_data = read_all(&mut initrd_file).expect("Failed to read kernel file into memory");
    }

    boot_linux_unchecked(
        handle,
        system_table,
        kernel_data,
        &config.cmdline,
        initrd_data,
    )
    .status()
}
