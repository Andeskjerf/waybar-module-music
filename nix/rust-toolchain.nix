_: {
  perSystem = {
    inputs',
    lib,
    ...
  }: let
    inherit (lib) types mkOption;
    inherit (inputs') fenix;
  in {
    options.waybar-module-music.rust-toolchain = mkOption {
      default = fenix.packages.fromToolchainFile {
        file = ../rust-toolchain.toml;
        sha256 = "sha256-vra6TkHITpwRyA5oBKAHSX0Mi6CBDNQD+ryPSpxFsfg=";
      };
      type = types.package;
      description = "The rust Toolchain we Use";
    };
  };
}
