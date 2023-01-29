# sd-stub-rs

This is a work-in-progress reimplementation of the systemd-stub in Rust. It is
supposed to eventually replace the existing systemd-stub in the upstream
systemd tree. The stub is the ideal playground for experimentation with Rust
within systemd because it relatvely isolated and corresponds to a leaf in the
systemd dependency tree.

The repository contains Nix code to set up a reproducible development
environment and for VM integration testing. However, because the stub should
eventually be integrated into the systemd build system, this will NOT be the
final build system. 

To make starting easier, the source code of the
[Lanzaboote](https://github.com/nix-community/lanzaboote) stub is copied as it
contains a lot of useful code that we can build from. The final behaviour of
the reimplementation of the systemd-stub, however, will significantly differ
from the Lanzaboote stub.

Once we find a better home for this repository it can be moved there.

## Goals

  - Replace the existing systemd-stub.
  - Feature parity and compatibility with the existing systemd-stub. The Rust
    stub is supposed to be a drop-in replacement.
  - Integrate this implementation with the systemd build system so the Rust
    stub can be bundled and shipped together with the rest of systemd.

## Non-Goals

  - Create anything distro-specific. The stub should remain as general-purpose
    as the current systemd-stub.
  - Develop new stub features. The first goal is to reach fearture partiy with
    the existing stub. Experiments with new behaviours may be done in a fork.
