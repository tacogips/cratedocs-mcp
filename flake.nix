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
        
        # Create a modified buildRustPackage that skips the problematic steps
        buildRustPackageCustom = args: pkgs.rustPlatform.buildRustPackage (args // {
          # These options help bypass the workspace inheritance issues
          dontFixCargo = true;
          cargoLockCheck = false;
          doCheck = false;
          
          # Provide a custom preBuild step that fixes the rmcp-macros issue
          preBuild = ''${pkgs.lib.optionalString (args ? preBuild) "${args.preBuild}"}
            if [ -d "$NIX_BUILD_TOP/source/vendor/rmcp" ]; then
              echo "Fixing rmcp-macros dependency..."
              find $NIX_BUILD_TOP/source/vendor -name "Cargo.toml" -type f -exec sed -i 's/rmcp-macros = { version = "0.1" }/rmcp-macros = "0.1"/g' {} \;
            fi
          '';
        });
      in {
        # Development shell with Rust toolchain
        devShells.default = pkgs.mkShell {
          packages = [
            rust-toolchain
            pkgs.nixpkgs-fmt
            pkgs.openssl
            pkgs.pkg-config
            pkgs.nodejs
            pkgs.nodePackages.npm
            pkgs.go-task
          ];
          
          # Add OpenSSL configuration
          shellHook = ''
            export OPENSSL_DIR=${pkgs.openssl.dev}
            export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
            export OPENSSL_INCLUDE_DIR=${pkgs.openssl.dev}/include
            echo "Shell loaded successfully with OpenSSL configuration"
          '';
        };
        
        # Simple package definition
        packages.default = buildRustPackageCustom {
          pname = "cratedocs-mcp";
          version = "0.1.0";
          src = ./.;
          
          # Basic cargo lock configuration
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };
          
          # Enable Git fetching with CLI
          CARGO_NET_GIT_FETCH_WITH_CLI = "true";
          CARGO_TERM_VERBOSE = "true";
          
          nativeBuildInputs = [ 
            rust-toolchain 
            pkgs.pkg-config
          ];
          
          buildInputs = [
            pkgs.openssl
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
          
          # OpenSSL environment variables
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
        };
      }
    );
}