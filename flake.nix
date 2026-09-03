# SPDX-License-Identifier: Apache-2.0 OR MIT
#
# UNVERIFIED — DO NOT TREAT AS A SUPPORTED INSTALL PATH.
#
# This file was written on a machine with no nix available, so it has
# never been evaluated. It is checked in so it can be tested somewhere
# that has nix, not because it is known to work. Nothing in the README,
# docs/packaging.md or CI references it, and it is deliberately not run
# in CI: gating the pipeline on an expression nobody has evaluated would
# be asserting something we do not know.
#
# Before this becomes a claim it needs, at minimum:
#
#     nix flake check --all-systems
#     nix build
#     nix develop --command cargo test --tests
#
# If those pass, wire it into CI and say so then.
{
  description = "SSG — a secure-by-default static site generator built in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Honour rust-toolchain.toml so `nix develop` and CI agree about
        # the compiler. See docs/packaging.md for the MSRV policy.
        rustToolchain =
          pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = manifest.name;
          inherit (manifest) version;
          src = pkgs.lib.cleanSource ./.;

          # Build against the committed lockfile: the dependency set
          # every gate in this repository ran against.
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          nativeBuildInputs = [ rustToolchain pkgs.pkg-config ];
          buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          # The example-building suites need a dev server on
          # 127.0.0.1:3000 and about thirteen minutes; they are gated on
          # SSG_REQUIRE_EXAMPLES, which is deliberately not set here.
          # See docs/packaging.md.
          checkFlags = [ "--skip=example_outputs" ];

          # Man page and shell completions are generated from the CLI
          # definition rather than committed, so they are produced here
          # rather than copied.
          postBuild = ''
            cargo run --release --offline --example gen-artifacts -- target/dist
          '';

          postInstall = ''
            installManPage target/dist/man/ssg.1
            installShellCompletion \
              --bash target/dist/completions/ssg \
              --zsh  target/dist/completions/_ssg \
              --fish target/dist/completions/ssg.fish
          '';

          meta = with pkgs.lib; {
            inherit (manifest) description homepage;
            license = with licenses; [ asl20 mit ];
            mainProgram = "ssg";
            platforms = platforms.unix ++ platforms.windows;
          };
        };

        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.pkg-config
            pkgs.cargo-deny
            pkgs.cargo-vet
            pkgs.cargo-llvm-cov
            pkgs.mandoc # install-smoke lints the generated man page
            pkgs.zsh # install-smoke parses the zsh completion
            pkgs.fish # ... and the fish one
            pkgs.reuse
            pkgs.typos
          ];

          shellHook = ''
            echo "ssg dev shell — see DEVELOPMENT.md for the CI gate table"
          '';
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
