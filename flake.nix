{
  description = "Rust libraries for interacting with hardware from TUXEDO Computers";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    pre-commit-hooks,
    flake-utils,
    ...
  }: let
    supportedSystems = [
      "x86_64-linux"
    ];

    overlay = import ./nix/overlay.nix {
      inherit
        self
        nixpkgs
        ;
    };
  in
    flake-utils.lib.eachSystem supportedSystems (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          overlay
        ];
      };

      pre-commit-check = pre-commit-hooks.lib.${system}.run {
        src = self;
        hooks = {
          alejandra.enable = true;
          rustfmt.enable = true;
        };
      };

      devShell = pkgs.mkShell {
        name = "tuxedo-rs-devShell";
        inherit (pre-commit-check) shellHook;
        buildInputs = with pkgs; [
          fenix.packages.${system}.stable.toolchain
          meson
          ninja
          libadwaita
          gtk4
          pkg-config
          desktop-file-utils
          appstream-glib
        ];
      };
    in {
      devShells = {
        default = devShell;
        inherit devShell;
      };

      packages = rec {
        default = tailord;
        inherit
          (pkgs)
          tailord
          tailor_gui
          ;
      };

      checks = {
        formatting = pre-commit-check;
        inherit
          (pkgs)
          tailord
          tailor_gui
          ;
      };
    })
    // {
      overlays.default = overlay;

      nixosModules.default = {...}: {
        imports = [
          ./nix/module.nix
        ];
        nixpkgs.overlays = [
          overlay
        ];
      };
    };
}
