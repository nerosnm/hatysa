{
  description = "Hatysa Discord bot";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    fenix.url = "github:nix-community/fenix/main";
    fenix.inputs.nixpkgs.follows = "nixpkgs";

    cargo2nix.url = "github:cargo2nix/cargo2nix/unstable";
    cargo2nix.inputs.nixpkgs.follows = "nixpkgs";
    cargo2nix.inputs.flake-utils.follows = "flake-utils";
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , fenix
    , cargo2nix
    } @ inputs:
    flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [
        cargo2nix.overlays.default
        fenix.overlays.default

        (final: prev: {
          rust-toolchain =
            let
              inherit (final) fenix;
              inherit (final.lib.strings) fileContents;

              # Use the same stable version for the main toolchain as
              # specified in the `rust-toolchain` file, so it's always the
              # same for Nix and non-Nix users.
              stable = fenix.toolchainOf {
                channel = fileContents ./rust-toolchain;
                sha256 = "sha256-riZUc+R9V35c/9e8KJUE+8pzpXyl0lRXt3ZkKlxoY0g=";
              };
            in
            fenix.combine [
              # Nightly rustfmt first so it overrides the one from the main
              # toolchain.
              fenix.latest.rustfmt

              # Make sure to include the `rust-std` component, then include the
              # main toolchain last.
              stable.rust-std
              stable.toolchain
            ];
        })
      ];

      pkgs = import nixpkgs {
        inherit system overlays;
      };

      formatPkgs = with pkgs; [
        nixpkgs-fmt
      ];

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
          packages = [
            cargo2nix.packages."${system}".default
          ] ++ formatPkgs;
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
            rust-toolchain
            cargo2nix.packages."${system}".default
          ] ++ formatPkgs;
        };
      };

      formatter = pkgs.nixpkgs-fmt;

      checks = {
        format = pkgs.runCommand "check-nix-format" { buildInputs = formatPkgs; } ''
          ${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt --check ${./.}
          touch $out
        '';
      };
    });
}
