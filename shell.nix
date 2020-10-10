with import <nixpkgs> {};

let
  moz_overlay = import (builtins.fetchTarball
    "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz");
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  rustStable = nixpkgs.latest.rustChannels.stable.rust.override {
    targets = ["wasm32-unknown-unknown"];
    extensions =
      [ "rust-src" "rls-preview" "rustfmt-preview" "clippy-preview" ];
  };
in with nixpkgs;
stdenv.mkDerivation rec {
  name = "moz_overlay_shell";
  buildInputs = [
    libGL
    libudev
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    rustStable
    pkg-config
    openssl
    cargo-web
  ];
  RUST_BACKTRACE = 1;
  LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
}
