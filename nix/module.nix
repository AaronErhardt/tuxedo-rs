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
    services.tuxedo-rs = {
      enable = mkEnableOption ''
        Rust utilities for interacting with hardware from TUXEDO Computers.
      '';

      tailor_gui.enable = mkEnableOption ''
        Alternative to Tuxedo Control Center, written in Rust.
      '';

      tailor_cli.enable = mkEnableOption ''
        CLI for interacting with tailord.
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
            ExecStart = "${pkgs.tailord}/bin/tailord";
            Environment = "RUST_BACKTRACE=1";
            Restart = "on-failure";
          };
        };
      };

      services.dbus.packages = [pkgs.tailord];

      # NOTE: By setting mode, the files are copied and not symlinked
      environment = {
        etc = {
          "tailord/keyboard/default.json" = {
            source = ../tailord/default_configs/keyboard/default.json;
            mode = "644";
          };
          "tailord/fan/default.json" = {
            source = ../tailord/default_configs/fan/default.json;
            mode = "644";
          };
          "tailord/profiles/default.json" = {
            source = ../tailord/default_configs/profiles/default.json;
            mode = "644";
          };
        };
      };
    }
    {
      environment.systemPackages = mkIf cfg.tailor_gui.enable [pkgs.tailor_gui];
    }
    {
      environment.systemPackages = mkIf cfg.tailor_cli.enable [pkgs.tailor_cli];
    }
  ]);
}
