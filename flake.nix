{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    systems.url = "github:nix-systems/default";
    devshell.url = "github:numtide/devshell";

    # The tool ships its own flake so it can be built and developed standalone.
    spacetree.url = "path:./tooling/spacetree";
    spacetree.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, systems, devshell, spacetree, ... }@flakeInputs:

  let
    lib = nixpkgs.lib;
    eachSystem = lib.genAttrs (import systems);

    forSys = system: {
      pkgs = nixpkgs.legacyPackages.${system};
      repoPkgs = self.packages.${system};
      devshPkgs = devshell.legacyPackages.${system};
      spacetreePkgs = spacetree.packages.${system};
    };
  in
  {
    packages = eachSystem (system: with (forSys system); {
      spacetree = spacetreePkgs.spacetree;
    });

    devShells = eachSystem (system: with (forSys system); {
      default = devshPkgs.mkShell {
        packages = [
          # bd <https://beads.gascity.com/> 🤯
          (pkgs.beads.overrideAttrs {
            # On nixos-unstable, beads gets built and the checkPhase takes FOREVER to run.. @2026-09
            doCheck = false;
          })
          repoPkgs.spacetree
        ];
      };
    });
  };
}
