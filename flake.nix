{
  description = "Rust broker-v2 dev environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (import rust-overlay)
          ];
        };
      in {
        devShell = pkgs.mkShell rec {
          nativeBuildInputs = with pkgs; [
            (pkgs.rust-bin.stable.latest.default.override {
                  extensions = [ "rust-src" "cargo" "rustc" ];
            })
            gcc

            xorg.libX11
            wayland
            libxkbcommon
            libGL
            libGLU
          ] ++ (with pkgs.xorg; [
            libxcb
            libXcursor
            libXrandr
            libXi
            pkg-config
          ]);

          shellHook = ''
              export LD_LIBRARY_PATH=/run/opengl-driver/lib/:${pkgs.lib.makeLibraryPath nativeBuildInputs}
          '';

          RUST_SRC_PATH = "${pkgs.rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
          }}/lib/rustlib/src/rust/library";


          buildInputs = with pkgs; [
            openssl.dev
            glib.dev
            pkg-config

            clippy
            rust-analyzer
            just
          ];
        };
      }
    );
}
