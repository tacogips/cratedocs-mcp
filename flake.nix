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

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system: 
      let
        pkgs = nixpkgs.legacyPackages.${system};
        
        # Get Rust toolchain from fenix - with updated hash
        rust-toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-AJ6LX/Q/Er9kS15bn9iflkUwcgYqRQxiOIL2ToVAXaU=";
        };
      in {
        # Development shell with Rust toolchain
        devShells.default = pkgs.mkShell {
          packages = [
            rust-toolchain
            pkgs.nixpkgs-fmt
          ];
          
          shellHook = ''echo "Shell loaded successfully"'';
        };
        
        # Simple package definition
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "cratedocs-mcp";
          version = "0.1.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          
          nativeBuildInputs = [ rust-toolchain ];
          
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };
      }
    );
}