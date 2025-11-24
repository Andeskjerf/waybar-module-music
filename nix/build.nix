{inputs, ...}: {
  perSystem = {
    config,
    pkgs,
    lib,
    ...
  }: let
    cfg = config.waybar-module-music;
    craneLib = (inputs.crane.mkLib pkgs).overrideToolchain cfg.rust-toolchain;
    commonArgs = {
      src = craneLib.cleanCargoSource ../.;
      nativeBuildInputs = with pkgs; [
        gcc
        pkg-config
        llvmPackages_19.bintools
      ];
      buildInputs = with pkgs; [
        dbus
      ];
      RUSTFLAGS = "-Clink-self-contained=-linker";
      meta.mainProgram = "waybar-module-music";
    };
    # get only the build dependencies so we can reuse them.
    cargoArtifacts = craneLib.buildDepsOnly ({} // commonArgs);
    # build the crate itself
    waybar-module-music = craneLib.buildPackage ({
        inherit cargoArtifacts;
        doCheck = false; # don't run check phase when building
      }
      // commonArgs);
    clippy = craneLib.cargoClippy ({
        inherit cargoArtifacts;
        cargoClippyExtraArgs = "--all-features";
      }
      // commonArgs);
    rustfmt = craneLib.cargoFmt ({} // commonArgs);
  in {
    packages = {
      default = waybar-module-music;
      inherit waybar-module-music;
    };
    checks = {
      inherit clippy rustfmt;
    };
  };
}
