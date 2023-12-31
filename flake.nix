{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        craneLib = crane.lib.${system};
        crateInfo = craneLib.crateNameFromCargoToml { cargoToml = ./visualizer/Cargo.toml; };
        pkgs = nixpkgs.legacyPackages.${system};
        lib = pkgs.lib;
        guiInputs = with pkgs; with pkgs.xorg; [ libX11 libXcursor libXrandr libXi vulkan-loader libxkbcommon wayland ];
        buildInputs = with pkgs; [ pkg-config systemd alsa-lib ];
        LD_LIBRARY_PATH = lib.makeLibraryPath (buildInputs ++ guiInputs);
        commonEnvironment = {
          nativeBuildInputs = with pkgs; [
            pkg-config
            rustc
            cargo
            rust-analyzer
            bacon
          ];
          inherit buildInputs;
        };
        assetsFilter = path: _type: builtins.match ".*assets$" path != null;
        assetsOrCargo = path: type: (assetsFilter path type) || (craneLib.filterCargoSources path type);
      in
    {
      packages.default = craneLib.buildPackage (lib.recursiveUpdate commonEnvironment {
        pname = crateInfo.pname;
        version = crateInfo.version;
        nativeBuildInputs = with pkgs; [ makeWrapper ];
        src = lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = assetsOrCargo;
        };
        # src = ./.;
        
        postInstall = ''
          wrapProgram "$out/bin/${crateInfo.pname}" \
            --prefix LD_LIBRARY_PATH : "${LD_LIBRARY_PATH}"
        '';
        # doCheck = true;
      });
      
      devShell = pkgs.mkShell (lib.recursiveUpdate commonEnvironment {
        inherit LD_LIBRARY_PATH;
        shellHook = ''
          exec $SHELL
        '';
        nativeBuildInputs = with pkgs; [
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
          (pkgs.writeShellScriptBin "xclip" ''
            # xclip wrapper that strips our LD_LIBRARY_PATH out to prevent breaking the fragile snowflake C code
            LD_LIBRARY_PATH="" ${pkgs.xclip}/bin/xclip "$@"
          '')
          (pkgs.writeShellScriptBin "run" ''
            cargo --locked run --features bevy/dynamic_linking "$@"
          '')
          (pkgs.writeShellScriptBin "test" ''
            cargo --locked test --features bevy/dynamic_linking "$@"
          '')
          (pkgs.writeShellScriptBin "build" ''
            cargo --locked build --features bevy/dynamic_linking "$@"
          '')
        ];
      });
      
    });
}
