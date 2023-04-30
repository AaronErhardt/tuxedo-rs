{
  description = "Rust libraries for interacting with hardware from TUXEDO Computers";

  nixConfig = {
    extra-substituters = "https://tuxedo-rs.cachix.org";
    extra-trusted-public-keys = "tuxedo-rs.cachix.org-1:blECq3BtB0X84VUHZAxvSJx3esqsuRdm59j2PCaOZ4I=";
  };

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
        buildInputs =
          (with pkgs;
            with pkgs.rustPlatform.rust; [
              cargo
              rustc
              meson
              ninja
              libadwaita
              gtk4
              pkg-config
              desktop-file-utils
              appstream-glib
            ])
          ++ (with pre-commit-hooks.packages.${system}; [
            alejandra
            rustfmt
          ]);
        shellHook = ''
          ${self.checks.${system}.formatting.shellHook}
        '';
      };
    in {
      devShells = {
        default = devShell;
        inherit devShell;
      };

      packages = rec {
        default = tuxedo-rs;
        inherit
          (pkgs)
          tuxedo-rs
          tailor_gui
          ;
      };

      checks = {
        formatting = pre-commit-check;
        inherit
          (pkgs)
          tuxedo-rs
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
