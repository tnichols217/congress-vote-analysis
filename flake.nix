{
  description = "Typescripten";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      ...
    }@inputs:
    inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import inputs.nixpkgs { inherit system; };
        pkgs_static =
          (import inputs.nixpkgs {
            inherit system;
            static = true;
          }).pkgsMusl;

        treefmtconfig = inputs.treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs = {
            nixfmt.enable = true;
            toml-sort.enable = true;
            yamlfmt.enable = true;
            shellcheck.enable = true;
            shfmt.enable = true;
            prettier.enable = true;
          };
          settings.formatter.shellcheck.excludes = [ ".envrc" ];
        };
      in
      rec {
        devShells = rec {
          rs = pkgs.mkShell {
            packages = (
              with pkgs;
              [
                nil
                nixd
                fontconfig
                rustPlatform.rustLibSrc
                rust-analyzer
                librsvg
              ]
              ++ packages.build.nativeBuildInputs
            );
          };
          default = rs;
        };
        formatter = treefmtconfig.config.build.wrapper;
        checks = {
          formatter = treefmtconfig.config.build.check self;
        };
        packages = rec {
          build = (pkgs.callPackage ./nix/build.nix {});
          default = build;
        };
      }
    );
}
