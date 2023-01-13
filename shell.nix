{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Dependencies
    cargo
    clippy
    rust-analyzer
    rustc
    rustfmt
  ];

  RUST_SRC_DIR = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
