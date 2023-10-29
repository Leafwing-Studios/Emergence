{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    devenv.url = "github:cachix/devenv";
    nix2container.url = "github:nlewo/nix2container";
    nix2container.inputs.nixpkgs.follows = "nixpkgs";
    mk-shell-bin.url = "github:rrbutani/nix-mk-shell-bin";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devenv.flakeModule
      ];
      systems = ["x86_64-linux" "i686-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin"];

      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        cargoToml = builtins.fromTOML (builtins.readFile ./emergence_game/Cargo.toml);
        # https://github.com/bevyengine/bevy/blob/latest/docs/linux_dependencies.md#nix
        # https://github.com/NixOS/nixpkgs/blob/master/pkgs/games/jumpy/default.nix

        buildInputs = with pkgs; [
          zstd # may need to config pkg-config
          udev
          alsa-lib
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr # To use the x11 feature
          libxkbcommon
          wayland # To use the wayland feature
        ];

        env = {
          ZSTD_SYS_USE_PKG_CONFIG = true;
        };
        #  ++ lib.optionals stdenv.isDarwin [
        #     darwin.apple_sdk.frameworks.Cocoa
        #     rustPlatform.bindgenHook
        #   ];
        nativeBuildInputs = with pkgs; [
          pkg-config
          llvmPackages.bintools # To use lld linker
        ];
        # LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
        pname = cargoToml.package.name;
        version = cargoToml.package.version;
        ide = with pkgs;
          vscode-with-extensions.override {
            vscode = vscodium;
            vscodeExtensions = with vscode-extensions; [
              jnoortheen.nix-ide
              tamasfe.even-better-toml
              serayuzgur.crates
              mkhl.direnv
              rust-lang.rust-analyzer
              github.copilot-chat
              github.copilot
              vadimcn.vscode-lldb
            ];
          };
        lib = pkgs.lib;
        filterCargoSources = orig_path: type: let
          path = toString orig_path;
          base = baseNameOf path;
          parentDir = baseNameOf (dirOf path);

          matchesSuffix = lib.any (suffix: lib.hasSuffix suffix base) [
            # Keep rust sources
            ".rs"
            # Keep all toml files as they are commonly used to configure other
            # cargo-based tools
            ".toml"
          ];

          # Cargo.toml already captured above
          isCargoFile = base == "Cargo.lock";

          # .cargo/config.toml already captured above
          isCargoConfig = parentDir == ".cargo" && base == "config";
        in
          type == "directory" || matchesSuffix || isCargoFile || isCargoConfig;
        src = lib.cleanSourceWith {
          # Apply the default source cleaning from nixpkgs
          src = lib.cleanSource ./.;

          # Then add our own filter on top
          filter = filterCargoSources;
        };
        assets = ./emergence_game/assets;
        built = with pkgs;
          rustPlatform.buildRustPackage rec {
            inherit pname version buildInputs nativeBuildInputs env src;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            cargoBuildFlags = ["--bin" pname];

            postInstall = ''
              cp -r ${assets} $out/bin/assets
            '';

            postFixup = lib.optionalString stdenv.isLinux ''
              patchelf $out/bin/${pname} \
                --add-rpath ${lib.makeLibraryPath [vulkan-loader]}
            '';
          };
      in {
        # Per-system attributes can be defined here. The self' and inputs'
        # module parameters provide easy access to attributes of the same
        # system.

        # Equivalent to  inputs'.nixpkgs.legacyPackages.hello;
        packages.ide = ide;
        packages.default = built;
        packages.blender = pkgs.blender;
        devenv.shells.default = {
          name = cargoToml.package.name;

          imports = [
            # This is just like the imports in devenv.nix.
            # See https://devenv.sh/guides/using-with-flake-parts/#import-a-devenv-module
            # ./devenv-foo.nix
          ];

          # https://devenv.sh/reference/options/
          packages = buildInputs ++ nativeBuildInputs;
          languages.rust.enable = true;
          pre-commit.hooks = {
            alejandra.enable = true;
            rustfmt.enable = true;
          };
          difftastic.enable = true;
          scripts.ide.exec = ''
            ${ide}/bin/codium
          '';
          scripts.trunk-serve.exec = ''
            ${pkgs.trunk}/bin/trunk serve
          '';

          scripts.build-assets.exec = ''
            ${pkgs.blender}/bin/blender --background assets/assets.blend --python "convert.py"
          '';

          scripts.blender-open.exec = ''
            ${pkgs.blender}/bin/blender ./assets/assets.blend
          '';
        };
        formatter = pkgs.alejandra;
      };
      flake = {
        # The usual flake attributes can be defined here, including system-
        # agnostic ones like nixosModule and system-enumerating ones, although
        # those are more easily expressed in perSystem.
      };
    };
}
