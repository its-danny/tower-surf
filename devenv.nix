{
  pkgs,
  lib,
  ...
}: {
  packages = [
    pkgs.act # This requires a locally-running Docker instance to run.
    pkgs.cargo-expand
    pkgs.cargo-msrv
    pkgs.cocogitto
    pkgs.git
    pkgs.rustup
    pkgs.sd
  ];

  languages = {
    rust = {
      enable = true;
      channel = "stable";
    };
  };

  pre-commit.hooks = {
    # Nix

    alejandra.enable = true;

    # Git

    cocogitto = {
      enable = true;
      entry = "cog verify --file .git/COMMIT_EDITMSG";
      stages = ["commit-msg"];
      pass_filenames = false;
    };

    # Rust

    cargo-check.enable = true;
    rustfmt.enable = true;
    clippy.enable = true;

    test = {
      enable = true;
      entry = "cargo test --all-features";
      pass_filenames = false;
      stages = ["pre-push"];
    };
  };
}
