{
  outputs = { self, nixpkgs }: {
    devShells.x86_64-linux.default = 
      let pkgs = nixpkgs.legacyPackages.x86_64-linux;
      in pkgs.mkShell {
        buildInputs = with pkgs; [
          pkg-config
          alsa-lib
        ];
      };
  };
}