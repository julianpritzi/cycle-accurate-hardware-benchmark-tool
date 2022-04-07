with import <nixpkgs> { };

let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  rustpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };

  rustToolchain = (rustpkgs.rustChannelOf { rustToolchain = ./rust-toolchain; }).rust.override {
    targets = [ "riscv32imac-unknown-none-elf" ];
    extensions = [
      "rust-src"
      "llvm-tools-preview"
      "rust-analyzer-preview"
      "rustfmt-preview"
    ];
  };
in
stdenv.mkDerivation rec {
  name = "env";

  nativeBuildInputs = [
    git
    qemu
    rustToolchain
  ];
}
