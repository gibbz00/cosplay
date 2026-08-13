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

            packages = with pkgs; [ ];
          };
        };
    };
}
