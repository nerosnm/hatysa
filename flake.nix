{
  description = "Hatysa Discord bot";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    fenix.url = "github:nix-community/fenix/main";
    fenix.inputs.nixpkgs.follows = "nixpkgs";

    cargo2nix.url = "github:cargo2nix/cargo2nix";
    cargo2nix.inputs.flake-utils.follows = "flake-utils";
    cargo2nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , ...
    } @ inputs:
    let
      pkgsFor = system: import nixpkgs {
        inherit system;
        overlays = [
          inputs.cargo2nix.overlays.default
          inputs.fenix.overlays.default

          (final: prev: {
            rust-toolchain =
              let
                inherit (final.lib.strings) fileContents;

                stableFor = target: target.toolchainOf {
                  channel = fileContents ./rust-toolchain;
                  sha256 = "sha256-eMJethw5ZLrJHmoN2/l0bIyQjoTX1NsvalWSscTixpI=";
                };

                rustfmt = final.fenix.latest.rustfmt;
              in
              final.fenix.combine [
                rustfmt
                (stableFor final.fenix).toolchain
              ];
          })

          (final: prev: {
            cargo2nix = inputs.cargo2nix.packages.${system}.default;
          })
        ];
      };

      supportedSystems = with flake-utils.lib.system; [
        aarch64-darwin
        x86_64-darwin
        x86_64-linux
      ];

      inherit (flake-utils.lib) eachSystem;
    in
    eachSystem supportedSystems (system:
    let
      pkgs = pkgsFor system;

      rustPkgs = pkgs.rustBuilder.makePackageSet {
        packageFun = import ./Cargo.nix;
        rustToolchain = pkgs.rust-toolchain;
      };
    in
    rec
    {
      packages = rec {
        hatysa = (rustPkgs.workspace.hatysa { }).bin;
        default = hatysa;
      };

      apps = rec {
        hatysa = flake-utils.lib.mkApp {
          drv = packages.hatysa;
        };
        default = hatysa;
      };

      devShells = {
        default = rustPkgs.workspaceShell {
          name = "hatysa";
          packages = with pkgs; [
            cargo2nix
            nixpkgs-fmt
            libiconv
          ];
        };

        # Sometimes it's useful to have this extra devShell, because there's a
        # bit of a circular dependency in the normal setup. The `cargo2nix` CLI
        # has to be available in order to generate a `Cargo.nix`, which has to
        # exist to create `workspaceShell`.
        #
        # So if something happens (a version change, etc.) that causes the
        # `cargo2nix` Nix code not to be able to read `Cargo.nix`, the
        # `workspaceShell` can't be built, which means no `cargo2nix` CLI tool
        # will be accessible. In that scenario, use the `backup` devShell to
        # build the CLI tool and update `Cargo.nix`.
        backup = pkgs.mkShell {
          name = "backup";
          packages = with pkgs; [
            cargo2nix
            nixpkgs-fmt
            rust-toolchain
          ];
        };
      };

      formatter = pkgs.nixpkgs-fmt;

      checks = {
        format = pkgs.runCommand
          "check-nix-format"
          { buildInputs = [ pkgs.nixpkgs-fmt ]; }
          ''
            ${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt --check ${./.}
            touch $out
          '';
      };
    });
}
