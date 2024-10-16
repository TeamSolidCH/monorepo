{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.05";
		nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    systems.url = "github:nix-systems/default";
    devenv.url = "github:cachix/devenv";
    devenv.inputs.nixpkgs.follows = "nixpkgs";
  };

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  outputs = { self, nixpkgs, devenv, systems, ... } @ inputs:
    let
      forEachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      packages = forEachSystem (system: {
        devenv-up = self.devShells.${system}.default.config.procfileScript;

      });

      devShells = forEachSystem
        (system:
          let
            pkgs = nixpkgs.legacyPackages.${system};
						pkgs-unstable = inputs.nixpkgs-unstable.legacyPackages.${system};
						unstable-packages = with pkgs-unstable; [
								moon
						];
          in
          {
            default = devenv.lib.mkShell {
              inherit inputs pkgs;
              modules = [
                {
                  # https://devenv.sh/reference/options/
                  packages = with pkgs; [ 
                    nix-ld
                    openssl.dev
                    pkg-config
                    dbus.dev
                    postgresql
                    diesel-cli
									] ++ unstable-packages; 

                  env = {
                    # https://devenv.sh/reference/environment/
                    NIX_LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
                      pkgs.stdenv.cc.cc
                      pkgs.xz
                      pkgs.postgresql
                    ];
                    NIX_LD = pkgs.lib.fileContents "${pkgs.stdenv.cc}/nix-support/dynamic-linker";
                    OPENSSL_DEV=pkgs.openssl.dev;
                  };
                }
              ];
            };
          }
        );
    };
}
