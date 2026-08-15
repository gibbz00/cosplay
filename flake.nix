{
  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    parts = {
      url = "github:gibbz00/parts";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-parts.follows = "flake-parts";
    };
  };

  outputs =
    inputs@{ nixpkgs, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;

      imports = [
        inputs.parts.flakeModule.pre-commit
        inputs.parts.flakeModule.rust
      ];

      perSystem =
        { config, pkgs, ... }:
        {
          craneLibSrcPath = ./.;

          devShells.default = pkgs.mkShell {
            name = "cosplay";

            inputsFrom = [
              config.devShells.pre-commit
              config.devShells.rust
            ];

            shellHook =
              let
                wayland-xml = pkgs.fetchurl {
                  url = "https://gitlab.freedesktop.org/wayland/wayland/-/raw/main/protocol/wayland.xml";
                  sha256 = "sha256-zIYJh+VPjYXJQOl/oScMabbkrTH7z1p/ABB84RV/Xgc=";
                };
                xdg-shell-xml = pkgs.fetchurl {
                  url = "https://gitlab.freedesktop.org/wayland/wayland-protocols/-/raw/main/stable/xdg-shell/xdg-shell.xml";
                  sha256 = "sha256-e6f5yEc97uZ0yx8VShir0LsMwHJgT8BVsMFeRZ/Ex98=";
                };
              in
              ''
                ln -fs ${wayland-xml} "./crates/protocols/wayland/input/protocol.xml"
                ln -fs ${xdg-shell-xml} "./crates/protocols/xdg-shell/input/protocol.xml"
              '';
          };
        };
    };
}
