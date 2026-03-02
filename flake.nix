{
  description = "A command-line tool for downloading and managing YouTube audio content";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        htb = pkgs.rustPlatform.buildRustPackage {
          pname = "htb";
          version = "0.1.0";
          src = ./.;

          cargoHash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

          buildInputs = with pkgs; [
            sqlite
          ];

          postInstall = ''
            wrapProgram $out/bin/htb \
              --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.yt-dlp pkgs.ffmpeg ]}
          '';

          nativeBuildInputs = with pkgs; [
            pkg-config
            makeWrapper
          ];
        };
      in
      {
        packages.default = htb;
        packages.htb = htb;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rust-analyzer
            rustc
            cargo
            rustfmt
            clippy

            openssl
            yt-dlp
            ffmpeg
            sqlite

            pkg-config
          ];

          RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        };

        apps.default = flake-utils.lib.mkApp {
          drv = htb;
        };
      });
}
