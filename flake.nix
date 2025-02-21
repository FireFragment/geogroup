{
  description = "Rust broker-v2 dev environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixkgs";
    };
    crate2nix.url = "github:nix-community/crate2nix";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    crate2nix,
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

        buildRustCrateForPkgs = pkgs: pkgs.buildRustCrate.override {
          defaultCrateOverrides = pkgs.defaultCrateOverrides // {
            rav1e = attrs: {
              CARGO_ENCODED_RUSTFLAGS = "";
            };
          };
        };

        crate2nix-tools = pkgs.callPackage "${crate2nix}/tools.nix" {};

        generatedCargoNix = crate2nix-tools.generatedCargoNix {
            name = "geogroup";
            src = ./.;
        };

        cargoNix = pkgs.callPackage "${generatedCargoNix}/default.nix" {
          inherit buildRustCrateForPkgs;
        };

        # TODO: Is anything superflous here?
        runtimeLibs = with pkgs; [
          wayland
          libxkbcommon
          libGL
          libGLU
          fontconfig
        ] ++ (with pkgs.xorg; [
          libX11
          libxcb
          libXcursor
          libXrandr
          libXi
          pkg-config
        ]);
      in {
        packages.default = pkgs.symlinkJoin {
          name = "geogroup_gui";
          paths = [ cargoNix.workspaceMembers.geogroup_gui.build ];
          buildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram $out/bin/geogroup_gui \
              --suffix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath runtimeLibs}
          '';
        };
        #;

        devShell = pkgs.mkShell rec {
          nativeBuildInputs = [
            (pkgs.rust-bin.stable.latest.default.override {
                  extensions = [ "rust-src" "cargo" "rustc" ];
            })
            pkgs.gcc
          ] ++ runtimeLibs;

          shellHook = ''
              export LD_LIBRARY_PATH=/run/opengl-driver/lib/:${pkgs.lib.makeLibraryPath runtimeLibs}
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
