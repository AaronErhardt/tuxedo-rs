{self}: final: prev: let
  pkgs = prev;
  lib = final.lib;

  tuxedo-rs = pkgs.tuxedo-rs.overrideAttrs (oa: {
    src = self;
    version = ((lib.importTOML "${self}/tailord/Cargo.toml").package).version;
    cargoDeps = pkgs.rustPlatform.importCargoLock {
      lockFile = self + "/Cargo.lock";
    };
  });

  tailor-gui = pkgs.tailor-gui.overrideAttrs (oa: {
    src = self;
    version = ((lib.importTOML "${self}/tailor_gui/Cargo.toml").package).version;
    cargoDeps = pkgs.rustPlatform.importCargoLock {
      lockFile = self + "/tailor_gui/Cargo.lock";
    };
  });
in {
  inherit
    tuxedo-rs
    tailor-gui
    ;
}
