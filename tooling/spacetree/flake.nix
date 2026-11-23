{
  inputs = {
    systems.url = "github:nix-systems/default";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    devshell.url = "github:numtide/devshell";
  };

  outputs = { systems, nixpkgs, devshell, ... }:
    let
      lib = nixpkgs.lib;
      forSystems = lib.genAttrs (import systems);

      commonsForSys = system: {
        inherit system;
        pkgs = nixpkgs.legacyPackages.${system};
        devshellPkgs = devshell.legacyPackages.${system};
      };
    in
    {
      packages = forSystems (
        system: with commonsForSys system; {
          spacetree = pkgs.callPackage ./package.nix { };
          default = pkgs.callPackage ./package.nix { };
        }
      );

      # Enough to build and test the tool without reaching for the repo root flake.
      devShells = forSystems (
        system: with commonsForSys system; {
          default = devshellPkgs.mkShell {
            packages = with pkgs; [
              rustc # needed for editor/LSP support
              cargo
              rustPackages.clippy
              rustfmt
              bats
              jq
            ];
          };
        }
      );
    };
}
