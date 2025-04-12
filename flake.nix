# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{
  description = "Flake for the NIKU project.";

  inputs = {
    # Use `shallow=1` to avoid insane slow download times
    # TRACK: https://github.com/NixOS/nix/issues/10683

    nixpkgs.url = "github:NixOS/nixpkgs?shallow=1&ref=nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay?shallow=1&ref=master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    ...
  }: let
    allSystems = [
      "x86_64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];

    # This functions defines the package for all the targets defined at the list `allSystems`.
    forAllSystems = callback:
      nixpkgs.lib.genAttrs allSystems
      (system:
        callback {
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              rust-overlay.overlays.default
            ];
          };
        });

    buildPackages = pkgs:
      with pkgs; [
        rust-bin.stable.latest.default
      ];

    devPackages = pkgs:
      with pkgs; [
        bash
        shellcheck
        taplo
        shfmt
        nixd
        cargo-machete
        nodePackages.prettier

        (lib.hiPrio rust-bin.nightly."2025-04-10".rustfmt)

        addlicense
      ];
  in {
    formatter = forAllSystems ({pkgs}: pkgs.alejandra);

    devShells = forAllSystems ({pkgs}: {
      default = pkgs.mkShell {
        name = "Dev";

        nativeBuildInputs = devPackages pkgs ++ buildPackages pkgs;
      };

      build = pkgs.mkShell {
        name = "Build";

        nativeBuildInputs = buildPackages pkgs;
      };
    });
  };
}
