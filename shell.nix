{ pkgs ? import <nixpkgs> { }
,
}:
let
  libPath =
    with pkgs;
    lib.makeLibraryPath [
      libGL
      libxkbcommon
      wayland
    ];

  currentSystemIsMac = builtins.isList (builtins.match ".*darwin" (builtins.currentSystem));

  platformDeps =
    if currentSystemIsMac then
      [
        pkgs.rustup
      ]
    else
      [
        pkgs.gcc
        pkgs.rustup
      ];

  deps = with pkgs; [
    pkg-config
    glib
  ];

  env =
    if currentSystemIsMac then
      {
        shellHook = ''
          rustup update stable
          rustup default stable
        '';
      }
    else
      {
        RUST_LOG = "debug";
        RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
        LD_LIBRARY_PATH = libPath;
        shellHook = ''
          export PATH="$HOME/.cargo/bin:$PATH"
          rustup update stable
          rustup default stable
        '';
      };

in
{
  devShell =
    with pkgs;
    mkShell (
      {
        buildInputs = [
          nil
          nixd
        ]
        ++ platformDeps
        ++ deps;
      }
      // env
    );
}
