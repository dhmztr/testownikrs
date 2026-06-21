{
  description = "testownik-rs — quiz app in Rust (iced, Wayland)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "clippy"
            "rustfmt"
          ];
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
        ];

        # iced 0.12 używa wgpu; rfd wymaga xdg-portal przez dbus
        buildInputs = with pkgs; [
          # Wayland
          wayland
          wayland-protocols
          libxkbcommon

          # X11 fallback (iced obsługuje oba backendy)
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          xorg.libxcb
          xorg.xcbutilkeysyms

          # wgpu / GPU
          vulkan-loader
          vulkan-headers
          libGL

          # rfd (natywny file dialog przez xdg-desktop-portal)
          dbus
          glib

          # encoding_rs / ogólne
          openssl
        ];

        # Zmienne środowiskowe potrzebne w runtime na Waylandzie
        runtimeEnv = {
          WINIT_UNIX_BACKEND = "wayland";
          WGPU_BACKEND = "vulkan"; # zamień na "gl" jeśli brak Vulkana
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (
            with pkgs;
            [
              wayland
              vulkan-loader
              libGL
              libxkbcommon
            ]
          );
        };

      in
      {
        # ── Pakiet ──────────────────────────────────────────────────────────
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "testownik-rs";
          version = "1.0.0";

          src = pkgs.fetchFromGitHub {
            owner = "Lukidere";
            repo = "testownikrs";
            rev = "main"; # zastąp konkretnym commitem/tagiem
            hash = pkgs.lib.fakeHash; # uruchom raz, wklej prawidłowy hash
          };

          cargoLock.lockFile = ./Cargo.lock; # skopiuj Cargo.lock obok flake.nix

          inherit nativeBuildInputs buildInputs;

          env = runtimeEnv // {
            PKG_CONFIG_PATH = pkgs.lib.makeSearchPathOutput "dev" "lib/pkgconfig" buildInputs;
          };

          # statyczne zasoby (folder static/) muszą być obok binarki w runtime
          postInstall = ''
            cp -r $src/static $out/bin/static || true
          '';

          meta = with pkgs.lib; {
            description = "Testownik — aplikacja quizowa w Rust (iced GUI)";
            license = licenses.mit;
            platforms = platforms.linux;
          };
        };

        # ── Aplikacja (alias) ────────────────────────────────────────────────
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/testownik-rs";
        };

        # ── Dev shell ────────────────────────────────────────────────────────
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;
          env = runtimeEnv;

          shellHook = ''
            echo "testownik-rs dev shell"
            echo "cargo build --release"
          '';
        };
      }
    );
}
