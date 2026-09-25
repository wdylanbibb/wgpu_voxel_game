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
        runtimeInputs = with pkgs; [ cargo rustc trunk binaryen lld ];
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
        nativeBuildInputs = with pkgs; [ trunk binaryen lld ];

        buildPhase = ''
          runHook preBuild
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
