{
  description = "Dynamic player controls module for Waybar, using DBus & MPRIS";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs = {flake-parts, ...} @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      # this is where we define the operating systems & architectures
      # that are supported.
      systems = [
        "x86_64-linux"
      ];
      imports = [
        ./nix/shell.nix
        ./nix/rust-toolchain.nix
        ./nix/build.nix
      ];

      perSystem = {pkgs, ...}: {
        # our preferred formatter. Like rustfmt, alejandra is unforgiving
        formatter = pkgs.alejandra;
      };
    };
}
