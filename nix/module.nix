# TODO: Remove this after NixOS 23.11 release
{
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  cfg = config.services.tuxedo-rs;
in {
  options = {
    hardware.tuxedo-rs = {
      enable = mkEnableOption ''
        Rust utilities for interacting with hardware from TUXEDO Computers.
      '';

      tailor-gui.enable = mkEnableOption ''
        Alternative to Tuxedo Control Center, written in Rust.
      '';
    };
  };

  config = mkIf cfg.enable (mkMerge [
    {
      hardware.tuxedo-keyboard.enable = true;

      systemd = {
        services.tailord = {
          enable = lib.mkDefault true;
          description = "Tux Tailor hardware control service";
          after = ["systemd-logind.service"];
          wantedBy = ["multi-user.target"];

          serviceConfig = {
            Type = "dbus";
            BusName = "com.tux.Tailor";
            ExecStart = "${pkgs.tuxedo-rs}/bin/tailord";
            Environment = "RUST_BACKTRACE=1";
            Restart = "on-failure";
          };
        };
      };

      services.dbus.packages = [pkgs.tuxedo-rs];

      environment.systemPackages = [pkgs.tuxedo-rs];
    }
    (mkIf cfg.tailor-gui.enable {
      environment.systemPackages = [pkgs.tailor-gui];
    })
  ]);
}
