{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
  };

  outputs = { self, nixpkgs }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [ "aarch64-darwin" "x86_64-linux" ];

      traceyVersion = "1.3.0";
      traceySources = {
        aarch64-darwin = {
          url = "https://github.com/bearcove/tracey/releases/download/v${traceyVersion}/tracey-aarch64-apple-darwin.tar.xz";
          hash = "sha256-NltLMFbFiZAVJrVEAbI2NEMKkl/LEyf1zW9TVoY7INU=";
        };
        x86_64-linux = {
          url = "https://github.com/bearcove/tracey/releases/download/v${traceyVersion}/tracey-x86_64-unknown-linux-gnu.tar.xz";
          hash = "sha256-+8DCXEQyjMsJcLJQkJX/KUEvpyy7xrADbEzujBKCH0c=";
        };
      };
    in {
      overlays.default = final: prev: {
        unpm = final.rustPlatform.buildRustPackage {
          pname = "unpm";
          version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;
          src = self;
          cargoLock.lockFile = ./Cargo.lock;
        };

        tracey = let
          src = traceySources.${final.stdenv.hostPlatform.system}
            or (throw "tracey: unsupported system ${final.stdenv.hostPlatform.system}");
        in final.stdenv.mkDerivation {
          pname = "tracey";
          version = traceyVersion;
          src = final.fetchurl { inherit (src) url hash; };
          nativeBuildInputs = [ final.xz ];
          sourceRoot = ".";
          unpackPhase = ''
            tar xf $src
          '';
          installPhase = ''
            install -Dm755 tracey-*/tracey $out/bin/tracey
          '';
        };
      };

      packages = forAllSystems (system:
        let pkgs = import nixpkgs { inherit system; overlays = [ self.overlays.default ]; };
        in {
          default = pkgs.unpm;
          tracey = pkgs.tracey;
        }
      );

      devShells = forAllSystems (system:
        let pkgs = import nixpkgs { inherit system; overlays = [ self.overlays.default ]; };
        in {
          default = pkgs.mkShell {
            packages = [ pkgs.tracey ];
          };
        }
      );
    };
}
