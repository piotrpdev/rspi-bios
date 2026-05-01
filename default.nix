let
  nixpkgs = fetchTarball "https://github.com/NixOS/nixpkgs/tarball/release-25.11";
  pkgsNative = import nixpkgs {};
  pkgsRaspberryPi4B = import nixpkgs { crossSystem = { config = "aarch64-unknown-linux-gnu"; }; };

  rspiBiosPkg = { lib, rustPlatform }:
    let manifest = (lib.importTOML ./Cargo.toml).package;
    in rustPlatform.buildRustPackage (finalAttrs: {
      pname = manifest.name;
      version = manifest.version;

      src = lib.cleanSource ./.;
      cargoLock.lockFile = ./Cargo.lock;

      meta = {
        description = manifest.description;
        homepage = manifest.repository;
        license = lib.licenses.gpl3;
        maintainers = [{
          email = "piotrpdev@gmail.com";
          github = "piotrpdev";
          githubId = 99439005;
          name = "Piotr Płaczek";
          keys = [{ fingerprint = "B4A4 4577 30D6 B2D4 C51E  69F2 8F33 147A 6EF6 EAB6"; }];
        }];
      };
    });
in {
  rspi-bios-native = pkgsNative.callPackage rspiBiosPkg {};
  rspi-bios-raspberry-pi-4b = pkgsRaspberryPi4B.callPackage rspiBiosPkg {};
}
