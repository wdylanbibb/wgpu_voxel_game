{
  description = "wgpu voxel game development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      graphicsLibraries = with pkgs; [
        libx11
        libxcursor
        libxi
        libxrandr
        libxkbcommon
        wayland
        vulkan-loader
      ];
      # Trunk needs a wasm-bindgen CLI matching the crate version selected by
      # Cargo. Fetch it as a fixed Nix input so the sandboxed build never asks
      # Trunk to create a cache or download a tool at build time.
      wasmBindgenCli = pkgs.stdenvNoCC.mkDerivation {
        pname = "wasm-bindgen-cli";
        version = "0.2.127";
        src = pkgs.fetchurl {
          url = "https://github.com/rustwasm/wasm-bindgen/releases/download/0.2.127/wasm-bindgen-0.2.127-x86_64-unknown-linux-musl.tar.gz";
          sha256 = "0jk1q4yyv5d7pa7g45vdfhmfbir834vbih6cahihvymchpfagm31";
        };
        sourceRoot = "wasm-bindgen-0.2.127-x86_64-unknown-linux-musl";
        installPhase = ''
          install -Dm755 wasm-bindgen "$out/bin/wasm-bindgen"
          install -Dm755 wasm-bindgen-test-runner "$out/bin/wasm-bindgen-test-runner"
          install -Dm755 wasm2es6js "$out/bin/wasm2es6js"
        '';
      };
      nativeRunner = pkgs.writeShellApplication {
        name = "wgpu-voxel-game-native";
        runtimeInputs = with pkgs; [ cargo rustc pkg-config ];
        text = ''
          export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath graphicsLibraries}:/run/opengl-driver/lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
          exec cargo run "$@"
        '';
      };
      webRunner = pkgs.writeShellApplication {
        name = "wgpu-voxel-game-web";
        runtimeInputs = with pkgs; [ cargo rustc trunk binaryen lld wasmBindgenCli ];
        text = ''
          NO_COLOR=false exec trunk serve --open "$@"
        '';
      };
      nativePackage = pkgs.rustPlatform.buildRustPackage {
        pname = "wgpu-voxel-game";
        version = "0.1.0";
        src = ./.;

        cargoLock.lockFile = ./Cargo.lock;
        nativeBuildInputs = [ pkgs.makeWrapper ];

        postInstall = ''
          wrapProgram "$out/bin/wgpu-voxel-game" \
            --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath graphicsLibraries}:/run/opengl-driver/lib"
        '';
      };
      webPackage = pkgs.rustPlatform.buildRustPackage {
        pname = "wgpu-voxel-game-web";
        version = "0.1.0";
        src = ./.;

        cargoLock.lockFile = ./Cargo.lock;
        nativeBuildInputs = with pkgs; [ trunk binaryen lld wasmBindgenCli ];

        buildPhase = ''
          runHook preBuild
          export HOME="$NIX_BUILD_TOP/home"
          export XDG_CACHE_HOME="$HOME/.cache"
          mkdir -p "$XDG_CACHE_HOME"
          NO_COLOR=false trunk build --release --dist "$NIX_BUILD_TOP/dist"
          runHook postBuild
        '';

        installPhase = ''
          runHook preInstall
          mkdir -p "$out"
          cp -r "$NIX_BUILD_TOP/dist/." "$out/"
          runHook postInstall
        '';

        doCheck = false;
      };
    in {
      apps.${system} = {
        native = {
          type = "app";
          program = "${nativeRunner}/bin/wgpu-voxel-game-native";
        };
        web = {
          type = "app";
          program = "${webRunner}/bin/wgpu-voxel-game-web";
        };
      };

      packages.${system} = {
        native = nativePackage;
        web = webPackage;
        default = nativePackage;
      };

      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          cargo
          rustc
          pkg-config
          trunk
          binaryen
          lld

          libx11
          libxcursor
          libxi
          libxrandr
          libxkbcommon
          wayland
          vulkan-loader
        ];

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath graphicsLibraries
          + ":/run/opengl-driver/lib";
      };
    };
}
