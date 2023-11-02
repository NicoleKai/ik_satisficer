{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };
        lib = pkgs.lib;

        buildInputs = with pkgs; [
          udev alsa-lib vulkan-loader
          xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr # To use the x11 feature
          libxkbcommon wayland # To use the wayland feature
        ];


        commonEnvironment = {
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
          inherit buildInputs;
        };

        naersk' = pkgs.callPackage naersk { };
      in
      {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage (lib.recursiveUpdate commonEnvironment {
          src = ./.;
          #   postInstall = ''
          #     # Working directory is set to bin/ when running from `nix run`, so we do something a little dirty here...
          #     # TODO: fix this properly
          #     ln -s ${./assets} $out/bin/assets
          # '';
        });

        devShell = pkgs.mkShell (lib.recursiveUpdate commonEnvironment {
          # Defaults to Bash for some reason???
          shellHook = ''
            exec $SHELL
          '';
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
            bacon
            (pkgs.writeShellScriptBin "git" ''
              email=nicolekohm102@gmail.com
              name=NicoleKai
              exec ${pkgs.git}/bin/git -c user.name=$name \
                       -c user.email=$email \
                       -c author.name=$name \
                       -c author.email=$email \
                       -c commiter.name=$name \
                       -c commiter.email=$email "$@"
            '')            
           ];
        });
      }
    );
}
