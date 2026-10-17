{ pkgs, ... }:

{
  packages = with pkgs; [
    just

    cargo-audit
    cargo-deny
    cargo-hack
    cargo-release
    cargo-watch
  ];

  languages = {
    rust = {
      enable = true;
      channel = "stable";

      lld.enable = true;
    };
  };
}
