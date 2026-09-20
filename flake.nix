{
  description = "Spotify, native and fast";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # rust-toolchain.toml pins the compiler so local builds and CI agree.
    # This reads that file rather than restating the version here.
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      systems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ (import rust-overlay) ];
            }
          )
        );
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages =
            with pkgs;
            [
              (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              rust-analyzer
              pkg-config
            ]
            ++ lib.optionals stdenv.hostPlatform.isDarwin [
              apple-sdk
            ]
            ++ lib.optionals stdenv.hostPlatform.isLinux [
              alsa-lib
              libpulseaudio
              libxkbcommon
              wayland
              libGL
              libx11
              libxcursor
              libxi
              libxrandr
            ];
          # The GUI dlopens its Wayland, X11 and GL libraries at run time.
          LD_LIBRARY_PATH = pkgs.lib.optionalString pkgs.stdenv.hostPlatform.isLinux (
            pkgs.lib.makeLibraryPath (
              with pkgs;
              [
                libxkbcommon
                wayland
                libGL
                libx11
                libxcursor
                libxi
                libxrandr
              ]
            )
          );
        };
      });

      packages = forAllSystems (
        pkgs:
        let
          magicspot =
            let
              toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
              rustPlatform = pkgs.makeRustPlatform {
                cargo = toolchain;
                rustc = toolchain;
              };
              runtimeLibs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux (
                with pkgs;
                [
                  libxkbcommon
                  wayland
                  libGL
                  libx11
                  libxcursor
                  libxi
                  libxrandr
                ]
              );
            in
            rustPlatform.buildRustPackage {
              pname = "magicspot";
              version = (pkgs.lib.importTOML ./Cargo.toml).package.version;
              src = self;

              # The lock file contains git dependencies. fetchCargoVendor includes
              # them in the fixed-output dependency tree, unlike cargoLock alone.
              cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
                pname = "magicspot";
                version = (pkgs.lib.importTOML ./Cargo.toml).package.version;
                src = self;
                hash = "sha256-m3mc9NppLyUkKNXv/U0NZOdLUC6CAi7+LUqfsc4/q30=";
              };

              nativeBuildInputs =
                with pkgs;
                [
                  pkg-config
                ]
                ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [ makeWrapper ];
              buildInputs =
                pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux (
                  with pkgs;
                  [
                    alsa-lib
                    libpulseaudio
                    libGL
                    libx11
                  ]
                )
                ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [ pkgs.apple-sdk ];

              # The GUI dlopens its Wayland, X11 and GL libraries at run time.
              postFixup = pkgs.lib.optionalString pkgs.stdenv.hostPlatform.isLinux ''
                wrapProgram $out/bin/magicspot \
                  --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath runtimeLibs}
              '';

              postInstall = pkgs.lib.optionalString pkgs.stdenv.hostPlatform.isLinux ''
                install -Dm644 packaging/applications/magicspot.desktop \
                  $out/share/applications/magicspot.desktop
                install -Dm644 packaging/icons/magicspot.svg \
                  $out/share/icons/hicolor/scalable/apps/magicspot.svg
              '';

              meta = {
                description = "Fast native Spotify client with local playback and Spotify Connect";
                homepage = "https://github.com/FallenG101/MagicSpot";
                license = pkgs.lib.licenses.mit;
                mainProgram = "magicspot";
              };
            };

          magicspot-app =
            let
              version = pkgs.lib.getVersion magicspot;
              build = pkgs.lib.head (pkgs.lib.splitString "-" version);
              icon =
                pkgs.runCommand "magicspot-icon"
                  {
                    nativeBuildInputs = [ pkgs.icnsify ];
                  }
                  ''
                    icnsify ${./packaging/macos/icon-1024.png} -o $out
                  '';
            in
            pkgs.runCommand "magicspot-app"
              {
                meta = {
                  description = "MagicSpot as a macOS app bundle";
                  homepage = "https://github.com/FallenG101/MagicSpot";
                  license = pkgs.lib.licenses.mit;
                  platforms = pkgs.lib.platforms.darwin;
                };
              }
              ''
                app="$out/Applications/MagicSpot.app/Contents"
                mkdir -p "$app/MacOS" "$app/Resources"
                cp ${magicspot}/bin/magicspot "$app/MacOS/magicspot"
                chmod 755 "$app/MacOS/magicspot"
                cp ${icon} "$app/Resources/magicspot.icns"
                sed -e "s/__VERSION__/${version}/g" -e "s/__BUILD__/${build}/g" \
                  ${./packaging/macos/Info.plist} > "$app/Info.plist"
                /usr/bin/codesign --force --sign - \
                  "$out/Applications/MagicSpot.app"
                /usr/bin/codesign --verify --strict \
                  "$out/Applications/MagicSpot.app"
              '';
        in
        {
          default = magicspot;
          inherit magicspot;
        }
        // pkgs.lib.optionalAttrs pkgs.stdenv.hostPlatform.isDarwin {
          inherit magicspot-app;
        }
      );

      formatter = forAllSystems (pkgs: pkgs.nixfmt-tree);
    };
}
