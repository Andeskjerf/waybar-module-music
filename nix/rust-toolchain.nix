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
        sha256 = "sha256-SDu4snEWjuZU475PERvu+iO50Mi39KVjqCeJeNvpguU=";
      };
      type = types.package;
      description = "The rust Toolchain we Use";
    };
  };
}
