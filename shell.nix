let
  sources = import ./nix/sources.nix;
  rust = import ./nix/rust.nix { inherit sources; };
  pkgs = import sources.nixpkgs { };
in pkgs.mkShell {
  buildInputs = [
    rust
    pkgs.cargo-fuzz
    pkgs.cargo-geiger
    pkgs.gcc
    pkgs.llvmPackages_12.lldb
    pkgs.p7zip
  ];
}
