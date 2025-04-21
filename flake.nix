{
  description = "cratedocs-mcp-forked";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Get Rust components from toolchain file
        rust-toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-ks0nMEGGXKrHnfv4Fku+vhQ7gx76ruv6Ij4fKZR3l78=";
        };

        # Create a fenix package with complete toolchain
        rust-package = fenix.packages.${system}.combine [
          rust-toolchain
          fenix.packages.${system}.latest.cargo
          fenix.packages.${system}.latest.rustc
        ];

        # Build the Rust package
        build-package = pkgs.rustPlatform.buildRustPackage {
          pname = "cratedocs-mcp";
          version = "0.1.0";
          src = ./.;
          cargoSha256 = pkgs.lib.fakeSha256; # Replace with actual hash after first build attempt

          nativeBuildInputs = [ rust-package ];

          buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

      in
      {
        # --- packages ---
        packages = {
          default = build-package;
          cratedocs-mcp = build-package;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = build-package;
        };

        formatter = pkgs.nixpkgs-fmt;

        # --- dev shell ---
        devShells.default = pkgs.mkShell {
          packages =
            with pkgs;
            [
              nixpkgs-fmt
              taplo-cli
              cargo-make
              cachix
            ]
            ++ [
              rust-package
              rust-toolchain
            ];

          shellHook = '''';
        };
      }
    );
}
