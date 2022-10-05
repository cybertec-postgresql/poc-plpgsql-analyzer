{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    naersk = { url = "github:nmattia/naersk"; inputs.nixpkgs.follows = "nixpkgs"; };
    oxalica = { url = "github:oxalica/rust-overlay"; };
  };

  outputs = { self, nixpkgs, naersk, oxalica, ... }@inputs:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ oxalica.overlay ];
      };

      callPackage = pkgs.lib.callPackageWith (pkgs // {
        inherit naerskLib rustTooling self;
        # inherit (rustTooling) rustc cargo rustPlatform;
      });

      naerskLib = pkgs.callPackage naersk {
        inherit (rustTooling) cargo rustc;
      };

      rustTooling = let
        rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
      in rec {
        inherit rust;
        rustc = rust;
        cargo = rust;
        rustPlatform = pkgs.makeRustPlatform {
          inherit rustc cargo;
        };
      };
    in {
      devShell.x86_64-linux = callPackage ({mkShell, rustTooling, nixpkgs-fmt, rustfmt}:
        mkShell {
          pname = "poc-plpgsql-analyzer";
          version = "0.0.1";
          buildInputs = [
            (rustTooling.rust.override { extensions = [ "rustfmt-preview" "rust-src" "rls-preview" "rust-analysis" ]; })
            nixpkgs-fmt
            rustfmt
          ];
        }) {};
      };
    }
