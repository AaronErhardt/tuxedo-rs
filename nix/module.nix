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
    };
  };

  config = mkIf cfg.enable {
    hardware.tuxedo-keyboard.enable = true;

    systemd.services.tailord = {
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

    services.dbus.packages = [pkgs.tailord];

    environment = {
      etc = {
        "tailord/keyboard".source = ../tailord/default_configs/keyboard;
        "tailord/fan".source = ../tailord/default_configs/fan;
        "tailord/profiles".source = ../tailord/default_configs/profiles;
        # FIXME: This has to be a symlink and it should be writable by tailord
        # "tailord/active_profile.json".source = ../tailord/default_configs/profiles/default.json;
      };

      systemPackages = mkIf cfg.tailor_gui.enable [
        pkgs.tailor_gui
      ];
    };
  };
}
