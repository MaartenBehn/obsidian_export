{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {

  name = "obsidian_export";
  RUSTC_VERSION = "stable";

  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
    export PATH=$PATH:~/go/bin/
    '';
 
  packages = with pkgs; [
    rustup
    nodejs_22
    hugo
    go
  ];
}


