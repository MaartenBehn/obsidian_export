{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {

  name = "obsidian_export";

  packages = with pkgs; [
    nodejs_22
  ];
}


