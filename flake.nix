{
  # Description of what this flake provides
  description = "htb - Download and track audio content from YouTube";

  # Input sources - these are the dependencies our flake needs
  inputs = {
    # The main Nix package repository
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # Flake utilities for common patterns
    flake-utils.url = "github:numtide/flake-utils";
  };

  # The main function that defines what this flake outputs
  outputs = { self, nixpkgs, flake-utils }:
    # This creates outputs for each supported system (Linux, macOS, etc.)
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Get the package set for our system
        pkgs = nixpkgs.legacyPackages.${system};

        # Define our Rust package
        htb = pkgs.rustPlatform.buildRustPackage {
          pname = "htb";
          version = "0.1.0";

          # Use the current directory as source
          src = ./.;

          # This will be generated automatically by Nix when you build
          # For now, we'll use a placeholder that Nix will help us fill in
          cargoHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

          # Runtime dependencies that need to be available when the program runs
          buildInputs = with pkgs; [
            # SQLite for rusqlite dependency
            sqlite
          ];

          # Make sure these tools are available at runtime
          # This wraps the binary to include these in PATH
          postInstall = ''
            wrapProgram $out/bin/htb \
              --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.yt-dlp pkgs.ffmpeg ]}
          '';

          nativeBuildInputs = with pkgs; [
            pkg-config
            makeWrapper  # Needed for wrapProgram
          ];
        };
      in
      {
        # Default package that gets built with `nix build`
        packages.default = htb;

        # Named package (can be built with `nix build .#htb`)
        packages.htb = htb;

        # Development shell for working on the project
        # Enter with `nix develop`
        devShells.default = pkgs.mkShell {
          # Tools available in the development environment
          buildInputs = with pkgs; [
            # Rust toolchain
            rust-analyzer
            rustc
            cargo
            rustfmt
            clippy

            # Runtime dependencies
            openssl
            yt-dlp
            ffmpeg
            sqlite

            # Development tools
            pkg-config
          ];

          # Environment variables
          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };

        # Apps that can be run with `nix run`
        apps.default = flake-utils.lib.mkApp {
          drv = htb;
        };
      });
}
