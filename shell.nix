with import <nixpkgs> {};

stdenv.mkDerivation rec {
  name = "rust-env";
  buildInputs = [
    libGL
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
  ];
  RUST_BACKTRACE = 1;
  LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
}
