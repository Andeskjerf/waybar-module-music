_: {
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    cfg = config.waybar-module-music;
  in
    with pkgs; {
      devShells.default = mkShell {
        buildInputs = [
          cfg.rust-toolchain
          dbus
          pkg-config # required for rust libraries to discover system libraries
          statix # nix linter. this way no one has to install it globally
          llvmPackages_19.bintools # get the wrapped ld linker for nix specific builds
        ];
        # prevent rustc from using the builtin lld linker.
        # This is because with nix, libraries are in different places (the nix store)
        # and thus might not be found.
        # This is why nix has wrapped linkers, so that they can directly tell the binary
        # where all of the necessary libraries are, by setting its rpath.
        RUSTFLAGS = ["-Clink-self-contained=-linker"];
      };
    };
}
