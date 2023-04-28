{ pkgs }:

let
  systemdStubModule = { config, ... }: {
    boot.loader.external = {
      enable = true;
      installHook = ''
        ${pkgs.systemdUkify}/lib/ukify \
          ${config.system.build.kernel}/${config.system.boot.loader.kernelFile} /
          ${config.system.build.initialRamdisk}/${config.system.boot.loader.initrdFile} /
          --cmdline="init=${config.system.build.toplevel}/init ${builtins.toString config.boot.kernelParams}"  \
          --os-release="${config.system.build.etc}/etc/os-release" \
          --output=uki.efi

        esp=${config.boot.loader.efi.efiSysMountPoint}

        ${pkgs.systemdUkify}/bin/bootctl install --esp-path="$esp"
        cp uki.efi "$esp"/EFI/Linux/
      '';
    };
  };

  mkTest = { name, machine ? { }, useSecureBoot ? true, testScript }: pkgs.nixosTest {
    inherit name testScript;
    nodes.machine = _: {
      imports = [
        systemdStubModule
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
  basic = mkTest {
    name = "sd-boot";
    testScript = ''
      machine.start()
      print(machine.succeed("bootctl status"))
    '';
  };
}
