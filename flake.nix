{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs?ref=nixpkgs-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    cargoNix = pkgs.callPackage ./Cargo.nix {};
  in {
    packages.${system}.lls = cargoNix.rootCrate.build;

    formatter.${system} = pkgs.alejandra;

    devShells.${system}.default = pkgs.mkShell {
      packages = [
        pkgs.rustc
        pkgs.cargo
        pkgs.rustfmt
        pkgs.clippy
        pkgs.crate2nix
      ];
      WNSEARCHDIR = "${pkgs.wordnet}/dict";
    };
  };
}
