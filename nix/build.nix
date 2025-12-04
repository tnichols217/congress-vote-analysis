{
  lib,
  rustPlatform,
  libxml2,
  libffi,
  zlib,
  llvm,
  llvmPackages,
  stdenv,
  musl,
  gcc,
  pkg-config,
  fontconfig,
  writeShellScriptBin,
  static ? false,
  ...
}:
rustPlatform.buildRustPackage {
  pname = "assignment-6";
  version = "0.0.1";
  src = ../src;

  cargoLock = {
    lockFile = ../src/Cargo.lock;
  };

  prebuild = ''
    export PKG_CONFIG_PATH="${fontconfig}/lib/pkgconfig$\{PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
  '';

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    fontconfig
  ];

  meta.mainProgram = "compiler";
}