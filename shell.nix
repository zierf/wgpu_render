{ pkgs ? import <nixpkgs> { } }:

with pkgs;

# Bevy NixShell
# https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md
mkShell rec {
  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs = [
    clang llvmPackages_18.bintools
    udev alsa-lib vulkan-loader
    xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr # To use the x11 feature
    libxkbcommon wayland # To use the wayland feature
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
}
