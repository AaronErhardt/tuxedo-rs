{
  self,
  nixpkgs,
}: final: prev:
with final.pkgs.stdenv; let
  # XXX: The nixos-22.11 rustPlatform is too old to build this.
  #TODO: We should use final.pkgs.rustPlatform when NixOS 23.05 has been released.
  pkgs = import nixpkgs {inherit (final.pkgs) system;};
  rustPlatform = pkgs.rustPlatform;

  tuxedo-rs = with pkgs.lib; let
    src = self;
  in
    rustPlatform.buildRustPackage {
      pname = "tuxedo-rs";
      inherit ((importTOML "${src}/tailord/Cargo.toml").package) version;

      inherit src;

      doCheck = false;

      cargoLock = {
        lockFile = self + "/Cargo.lock";
      };

      postInstall = ''
        mkdir -p $out/share/dbus-1/system.d
        cp ${self}/tailord/com.tux.Tailor.conf $out/share/dbus-1/system.d
      '';

      meta = with final.lib; {
        description = "Daemon handling fan, keyboard and general HW support for Tuxedo laptops (part of tuxedo-rs)";
        homepage = "https://github.com/AaronErhardt/tuxedo-rs";
        license = licenses.gpl2Only;
      };
    };

  tailor_gui = with pkgs.lib; let
    src = builtins.path {
      path = self + "/tailor_gui";
      name = "tailor_gui";
    };
  in
    mkDerivation {
      name = "tailor_gui";

      inherit ((importTOML (src + "/Cargo.toml")).package) version;

      inherit src;

      cargoDeps = rustPlatform.importCargoLock {
        lockFile = self + "/tailor_gui/Cargo.lock";
      };

      nativeBuildInputs = with rustPlatform;
        [
          rust.cargo
          rust.rustc
          cargoSetupHook
        ]
        ++ (with pkgs; [
          pkg-config
          desktop-file-utils
          appstream-glib
          makeWrapper
        ]);

      buildInputs = with pkgs; [
        meson
        ninja
        libadwaita
        gtk4
      ];

      postFixup = ''
        wrapProgram $out/bin/tailor_gui --set XDG_DATA_DIRS "$out/share/gsettings-schemas/tailor_gui"
      '';
    };
in {
  inherit
    tuxedo-rs
    tailor_gui
    ;
}
