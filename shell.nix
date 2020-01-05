with import <nixpkgs> {};

stdenv.mkDerivation rec {
  name = "rust-env";
  buildInputs = [
    libGL
  ];
  RUST_BACKTRACE = 1;
  LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
}
