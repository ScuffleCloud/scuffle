{
  description = "Scuffle";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    alejandra = {
      url = "github:kamadorueda/alejandra/4.0.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    alejandra,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };
      in {
        devShells.default = with pkgs; mkShell {
          name = "scuffle-shell";

          buildInputs = [
            git
            bazelisk
            stdenv
            bash
            openssl
            pkg-config
            ffmpeg-full
            llvmPackages_20.clangWithLibcAndBasicRtAndLibcxx
          ];

          shellHook = ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${stdenv.cc.cc.lib}/lib";
            export PKG_CONFIG_PATH="$PKG_CONFIG_PATH:${openssl.dev}/lib/pkgconfig:${ffmpeg-full.lib}/lib/pkgconfig";
            export LIBCLANG_PATH="${llvmPackages_20.libclang.lib}/lib"
          '';
        };

        formatter = alejandra.defaultPackage.${system};
      }
    );
}
