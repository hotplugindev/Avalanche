{
  description = "Avalanche test fixture";
  outputs = { self }:
    let
      system = "x86_64-linux";
    in
    {

      nixosConfigurations.testhost = {
        inherit system;
        pkgs = {
          stdenv = {
            hostPlatform = {
              inherit system;
            };
          };
        };
        config = {
          services = {
            pipewire = {
              enable = true;
            };
          };
          gb = {
            schemaVersion = 1;
            programs = {
              git = {
                enable = true;
              };
            };
            requires = {
              system = {
                audio = {
                  pipewire = [ "programs.system.gaming.steam" ];
                };
              };
            };
          };
          assertions = [
            {
              message = "test assertion passes";
              assertion = true;
            }
            {
              message = "test assertion fails";
              assertion = false;
            }
          ];
        };
        options = {
          services.pipewire.enable = {
            type = {
              description = "boolean";
            };
            default = false;
            description = "Whether to enable PipeWire.";
            internal = false;
            readOnly = false;
          };
          gb.programs.git.enable = {
            type = {
              description = "boolean";
            };
            default = false;
            description = "Whether to enable git.";
            internal = false;
            readOnly = false;
          };
        };
      };

      homeConfigurations."testuser@testhost" = {
        config = {
          programs = {
            git = {
              enable = true;
              userName = "Test User";
            };
          };
        };
        options = {
          programs.git.enable = {
            type = {
              description = "boolean";
            };
            default = false;
            description = "Whether to enable git.";
            internal = false;
            readOnly = false;
          };
        };
      };

      testValues = {
        bool = true;
        int = 42;
        float = 3.14;
        str = "hello";
        list = [ 1 2 3 ];
        attrs = { a = 1; b = "two"; };
        nested = { x = { y = { z = true; }; }; };
      };
    };
}
