{
  self,
  fenix,
}: final: prev:
with final.pkgs.stdenv; let
  pkgs = final.pkgs;

  rustToolchain = fenix.packages.${pkgs.system}.stable.toolchain;

  rustPlatform = pkgs.makeRustPlatform {
    cargo = rustToolchain;
    rustc = rustToolchain;
  };

  tailord = rustPlatform.buildRustPackage {
    pname = "tailord";
    version = "0.1.0";

    src = self;

    doCheck = false;

    cargoLock = {
      lockFile = "${self}/Cargo.lock";
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

  tailor_gui = mkDerivation {
    name = "tailor_gui";
    version = "0.1.0";

    src = builtins.path {
      path = "${self}/tailor_gui";
      name = "tailor_gui";
    };

    buildInputs = with pkgs; [
      meson
      ninja
      libadwaita
      gtk4
      rustToolchain
      pkg-config
      desktop-file-utils
      appstream-glib
      makeWrapper
    ];

    postFixup = ''
      wrapProgram $out/bin/tailor_gui --set XDG_DATA_DIRS "$out/share/gsettings-schemas/tailor_gui"
    '';
  };
in {
  inherit
    tailord
    tailor_gui
    ;
}
