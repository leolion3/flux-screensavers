{
  description = "Flux Screensavers";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    crane,
    rust-overlay,
  }: let
    sdl2StaticDrv = {SDL2, lib, targetPlatform, ... }:
      SDL2.overrideAttrs (old: {
        name = "SDL2-static-${old.version}";
        dontDisableStatic = true;

        # When statically linking for Windows, rust-sdl2 expects the library to be called 'SDL2-static'.
        # https://github.com/Rust-SDL2/rust-sdl2/blob/ffa4eb0b15439463561014f2d3c9d9171059d492/sdl2-sys/build.rs#L237-L238
        postInstall = lib.concatLines [
            old.postInstall
            (lib.optionalString targetPlatform.isWindows ''
              mv $out/lib/libSDL2.a $out/lib/libSDL2-static.a
              mv $out/lib/libSDL2.dll.a $out/lib/libSDL2-static.dll.a
            '')
          ];
      });

    extensions = [
      "rust-src"
      "cargo"
      "rustc"
      "rls"
      "rust-analyzer"
      "rustfmt"
    ];
  in
    nixpkgs.lib.foldAttrs nixpkgs.lib.recursiveUpdate {} [
      (flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"] (
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [(import rust-overlay)];
          };

          inherit (pkgs) stdenv lib;

          rustToolchain = pkgs.rust-bin.nightly.latest.default.override {inherit extensions;};

          craneLib = (crane.mkLib pkgs).overrideScope' (final: prev: {
            rustc = rustToolchain;
            cargo = rustToolchain;
            rustfmt = rustToolchain;
          });

          sdl2Static = pkgs.callPackage sdl2StaticDrv {};

          x11BuildInputs = [
            pkgs.xorg.libX11
            pkgs.xorg.libXext
            pkgs.xorg.libXrandr
            pkgs.xorg.libXi
            pkgs.xorg.libXScrnSaver
            pkgs.xorg.libXcursor
          ];

          darwinBuildInputs = with pkgs.darwin.apple_sdk.frameworks; [
            AppKit
            ApplicationServices
            AudioToolbox
            Cocoa
            CoreAudio
            CoreFoundation
            CoreGraphics
            CoreHaptics
            CoreText
            CoreVideo
            ForceFeedback
            Foundation
            GameController
            Metal
            OpenGL
            QuartzCore
          ];
        in rec {
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              pkg-config
              rustToolchain
              sdl2Static
              cargo-outdated
              fontconfig
              cmake
              alejandra
            ] ++ lib.optionals stdenv.isLinux x11BuildInputs
              ++ lib.optionals stdenv.isDarwin darwinBuildInputs;
          };

          packages.default = packages.flux-wrapped;

          packages.flux = craneLib.buildPackage {
            src = ./windows;
            release = true;

            buildInputs = with pkgs; [
              pkg-config
              fontconfig
              cmake
              sdl2Static
            ] ++ lib.optionals stdenv.isLinux x11BuildInputs
              ++ lib.optionals stdenv.isDarwin darwinBuildInputs;
          };

          packages.flux-wrapped = let 
            runtimeLibraries = with pkgs; [
              wayland
              wayland-protocols
              xorg.libX11
              xorg.libXcursor
              xorg.libXrandr
              xorg.libXi
              libGL
            ];
            in pkgs.stdenvNoCC.mkDerivation {
              name = "flux-screensaver-wrapped";
              inherit (packages.flux) version;
              nativeBuildInputs = [pkgs.makeWrapper];
              buildCommand = ''
                mkdir -p $out/bin
                cp ${packages.flux}/bin/Flux $out/bin
                wrapProgram $out/bin/Flux \
                  --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath runtimeLibraries}
              '';
              passthru.unwrapped = packages.flux;
            };
        }
      ))
      (flake-utils.lib.eachSystem ["x86_64-linux" "aarch64-linux"] (system: let
        target = "x86_64-pc-windows-gnu";

        pkgs = import nixpkgs {
          inherit system;
          crossSystem.config = "x86_64-w64-mingw32";
          overlays = [(import rust-overlay)];
        };

        rustToolchain = pkgs.pkgsBuildHost.rust-bin.nightly.latest.default.override {
          inherit extensions;
          targets = [target];
        };

        craneLib = (crane.mkLib pkgs).overrideScope' (final: prev: {
          rustc = rustToolchain;
          cargo = rustToolchain;
          rustfmt = rustToolchain;
        });

        sdl2Static = pkgs.callPackage sdl2StaticDrv {};
      in {
        devShells.cross = pkgs.pkgsBuildHost.mkShell {
          # inputsFrom = [packages.default];

          packages = with pkgs.pkgsBuildHost; [
            rustToolchain
            sdl2Static
            cargo-outdated
            fontconfig
            cmake
            alejandra
            nsis
          ];

          RUSTFLAGS = "-L ${sdl2Static}/lib";
        };

        packages.windows = rec {
          default = installer;

          installer = pkgs.stdenvNoCC.mkDerivation {
            name = "flux-screensaver-installer";
            version = flux.version;
            src = ./windows/installer;

            buildInputs = with pkgs.pkgsBuildHost; [nsis flux];

            installPhase = ''
              mkdir -p $out/bin
              ${pkgs.pkgsBuildHost.nsis}/bin/makensis \
                -DDSTDIR=${flux}/bin/ \
                -DOUTDIR=$out/bin \
                -DVERSION=${flux.version} \
                $src/setup.nsi
            '';
          };

          flux = craneLib.buildPackage {
            src = ./windows;
            release = true;
            doCheck = false;

            buildInputs = [
              pkgs.windows.pthreads
              pkgs.windows.mingw_w64_pthreads
              sdl2Static
            ];

            CARGO_BUILD_TARGET = target;
            CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "${pkgs.stdenv.cc.targetPrefix}cc";
            # Link to the static SDL2 library and export the static GPU preference symbols
            RUSTFLAGS = "-L ${sdl2Static}/lib -Zexport-executable-symbols";

            # Change the extension to .scr (Windows screensaver)
            postInstall = ''
              if [[ $out != *"deps"* ]]; then
                cp $out/bin/Flux.exe "$out/bin/Flux.scr"
              fi
            '';
          };
        };
      }))
    ];
}
