{
  self,
  nixpkgs,
}: final: prev: let
  # To make this usable with NixOS < 23.11,
  # we fall back to nixpkgs from the flake inputs.
  pkgs =
    if builtins.hasAttr "tuxedo-rs" prev
    then prev
    else nixpkgs.legacyPackages.${final.system};
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
    sourceRoot = "source/tailor_gui";
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
