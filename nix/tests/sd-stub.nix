{ pkgs }:

let
  sd-stub-path = "${pkgs.systemd}/lib/systemd/boot/efi/linuxx64.efi.stub";
  sd-stub-rs-path = "${pkgs.sd-stub-rs}/bin/sd-stub-rs.efi";

  sd-stub-module = { config, lib, pkgs, ... }:
    let
      cfg = config.boot.sd-stub;
    in
    {
      options.boot.sd-stub = {
        path = lib.mkOption {
          type = lib.types.path;
          description = "Path to the systemd stub to use.";
          default = sd-stub-rs-path;
        };
      };

      config = {
        boot.bootspec.enable = true;
        boot.loader.external = {
          enable = true;
          installHook =
            let
              runtimeInputs = with pkgs; [ jq systemd binutils ];
            in
            pkgs.writeShellScript "install-uki" ''
              set -euo pipefail
              export PATH="${lib.makeBinPath runtimeInputs}:$PATH"

              boot_json=/nix/var/nix/profiles/system-1-link/boot.json
              kernel=$(jq -r '.v1.kernel' "$boot_json")
              initrd=$(jq -r '.v1.initrd' "$boot_json")
              init=$(jq -r '.v1.init' "$boot_json")

              ${pkgs.systemdUkify}/lib/systemd/ukify \
                "$kernel" \
                "$initrd" \
                --stub=${cfg.path} \
                --cmdline="init=$init ${builtins.toString config.boot.kernelParams}" \
                --os-release="@${config.system.build.etc}/etc/os-release" \
                --output=uki.efi

              esp=${config.boot.loader.efi.efiSysMountPoint}

              bootctl install --esp-path="$esp"
              cp uki.efi "$esp"/EFI/Linux/
            '';
        };
      };
    };

  mkTest = { name, machine ? { }, useSecureBoot ? false, testScript }: pkgs.nixosTest {
    inherit name testScript;
    nodes.machine = _: {
      imports = [
        sd-stub-module
        machine
      ];

      virtualisation = {
        useBootLoader = true;
        useEFIBoot = true;

        inherit useSecureBoot;
      };

      boot.loader.efi = {
        canTouchEfiVariables = true;
      };
    };
  };
in
{
  sd-stub = mkTest {
    name = "sd-stub";
    machine = _: {
      boot.sd-stub.path = sd-stub-path;
    };
    testScript = ''
      machine.start()
      print(machine.succeed("bootctl status"))
    '';
  };

  # sd-stub-rs = mkTest {
  #   name = "sd-stub-rs";
  #   testScript = ''
  #     machine.start()
  #     print(machine.succeed("bootctl status"))
  #   '';
  # };
}
