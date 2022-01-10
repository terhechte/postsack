with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "postsack-env";
  buildInputs = [
    rustc cargo xorg.libxcb
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath [
    xorg.libxcb
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    libglvnd
  ];
}
