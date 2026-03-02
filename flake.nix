{
  description = "A command-line tool for downloading and managing YouTube audio content";

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
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Extract common dependencies
        runtimeDeps = with pkgs; [
          yt-dlp
          ffmpeg
        ];

        buildDeps = with pkgs; [
          openssl
          sqlite
        ];

        htb = pkgs.rustPlatform.buildRustPackage {
          pname = "htb";
          version = "0.1.0";
          src = ./.;

          cargoHash = "sha256-4oINRkAPGADZGEzRaYs45H1wuNBX4Bly9nnpni/6wbQ=";

          buildInputs = buildDeps;

          nativeBuildInputs = with pkgs; [
            pkg-config
            makeWrapper
          ];

          postInstall = ''
            wrapProgram $out/bin/htb \
              --prefix PATH : ${pkgs.lib.makeBinPath runtimeDeps}
          '';

          meta = with pkgs.lib; {
            description = "A command-line tool for downloading and managing YouTube audio content";
            homepage = "https://github.com/gabrielopesantos/htb";
            license = licenses.mit;
            mainProgram = "htb";
          };
        };
      in
      {
        packages.default = htb;
        packages.htb = htb;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Use rust-bin for more control over Rust toolchain
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" ];
            })

            pkg-config
          ] ++ buildDeps ++ runtimeDeps;

          # Environment variables for development
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          RUST_BACKTRACE = "1";

          shellHook = ''
            echo "htb development environment"
            echo "Run 'cargo run -- --help' to get started"
          '';
        };

        apps.default = flake-utils.lib.mkApp {
          drv = htb;
        };

        # Add a formatter for 'nix fmt'
        formatter = pkgs.nixpkgs-fmt;
      }
    );
}