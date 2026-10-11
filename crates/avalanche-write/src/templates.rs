use avalanche_model::Scope;
use avalanche_source::{Binding, Expr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramSpec {
    pub scope: Scope,
    pub category: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilitySpec {
    pub scope: Scope,
    pub domain: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileSpec {
    pub scope: Scope,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostSpec {
    pub name: String,
    pub system: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserSpec {
    pub name: String,
    pub full_name: String,
    pub email: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitSpec {
    pub host_name: String,
    pub host_system: String,
    pub user_name: String,
    pub user_full_name: String,
    pub user_email: String,
}

impl ProgramSpec {
    pub fn validate(&self) -> Result<(), String> {
        validate_identifier(&self.name, "program name")?;
        match self.scope {
            Scope::System => {
                if !self.category.is_empty() {
                    return Err("system programs cannot have a category".to_string());
                }
            }
            Scope::Home => {
                if !self.category.is_empty() {
                    validate_identifier(&self.category, "program category")?;
                }
            }
        }
        Ok(())
    }

    pub fn option_prefix(&self) -> String {
        match self.scope {
            Scope::System => format!("gb.programs.system.{}", self.name),
            Scope::Home => {
                if self.category.is_empty() {
                    format!("gb.home.programs.{}", self.name)
                } else {
                    format!("gb.home.programs.{}.{}", self.category, self.name)
                }
            }
        }
    }

    pub fn module_path(&self) -> String {
        match self.scope {
            Scope::System => format!("modules/programs/system/{}.nix", self.name),
            Scope::Home => {
                if self.category.is_empty() {
                    format!("modules/programs/home/{}.nix", self.name)
                } else {
                    format!("modules/programs/home/{}/{}.nix", self.category, self.name)
                }
            }
        }
    }

    pub fn aggregate_path(&self) -> String {
        aggregate_path(self.scope)
    }

    pub fn relative_import(&self) -> String {
        relative_from_aggregate(&self.module_path())
    }

    pub fn category_default_path(&self) -> Option<String> {
        if self.scope == Scope::Home && !self.category.is_empty() {
            Some(format!(
                "modules/programs/home/{}/default.nix",
                self.category
            ))
        } else {
            None
        }
    }

    pub fn category_relative_import(&self) -> Option<String> {
        self.category_default_path()
            .map(|p| relative_from_aggregate(&p))
    }
}

impl CapabilitySpec {
    pub fn validate(&self) -> Result<(), String> {
        validate_dotted(&self.domain, "capability domain")?;
        validate_identifier(&self.name, "capability name")?;
        Ok(())
    }

    pub fn request_path(&self) -> String {
        format!("gb.requires.{}.{}.{}", self.scope, self.domain, self.name)
    }

    pub fn module_path(&self) -> String {
        format!(
            "modules/capabilities/{}/{}{}.nix",
            scope_dir(self.scope),
            domain_dir(&self.domain),
            self.name
        )
    }

    pub fn aggregate_path(&self) -> String {
        aggregate_path(self.scope)
    }

    pub fn relative_import(&self) -> String {
        relative_from_aggregate(&self.module_path())
    }
}

impl ProfileSpec {
    pub fn validate(&self) -> Result<(), String> {
        validate_identifier(&self.name, "profile name")?;
        Ok(())
    }

    pub fn requester(&self) -> String {
        format!("profiles.{}.{}", self.scope, self.name)
    }

    pub fn module_path(&self) -> String {
        format!(
            "modules/profiles/{}/{}.nix",
            scope_dir(self.scope),
            self.name
        )
    }

    pub fn aggregate_path(&self) -> String {
        aggregate_path(self.scope)
    }

    pub fn relative_import(&self) -> String {
        relative_from_aggregate(&self.module_path())
    }
}

impl HostSpec {
    pub fn validate(&self) -> Result<(), String> {
        validate_identifier(&self.name, "host name")?;
        validate_component(&self.system, "host system")?;
        Ok(())
    }

    pub fn module_path(&self) -> String {
        format!("hosts/{}/default.nix", self.name)
    }
}

impl UserSpec {
    pub fn validate(&self) -> Result<(), String> {
        validate_identifier(&self.name, "user name")?;
        Ok(())
    }

    pub fn module_path(&self) -> String {
        format!("users/{}.nix", self.name)
    }
}

impl InitSpec {
    pub fn validate(&self) -> Result<(), String> {
        validate_identifier(&self.host_name, "host name")?;
        validate_component(&self.host_system, "host system")?;
        validate_identifier(&self.user_name, "user name")?;
        Ok(())
    }

    pub fn host_spec(&self) -> HostSpec {
        HostSpec {
            name: self.host_name.clone(),
            system: self.host_system.clone(),
        }
    }

    pub fn user_spec(&self) -> UserSpec {
        UserSpec {
            name: self.user_name.clone(),
            full_name: self.user_full_name.clone(),
            email: self.user_email.clone(),
        }
    }
}

pub fn validate_component(value: &str, what: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{what} must not be empty"));
    }
    if value == "." || value == ".." {
        return Err(format!("{what} must not be '{value}'"));
    }
    if value.chars().any(|c| c.is_whitespace()) {
        return Err(format!("{what} must not contain whitespace"));
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(format!("{what} contains unsupported characters"));
    }
    Ok(())
}

pub fn validate_dotted(value: &str, what: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{what} must not be empty"));
    }
    for segment in value.split('.') {
        validate_component(segment, what)?;
    }
    Ok(())
}

pub fn validate_identifier(value: &str, what: &str) -> Result<(), String> {
    if value.is_empty() {
        return Err(format!("{what} must not be empty"));
    }
    let mut chars = value.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '_') {
        return Err(format!("{what} must start with a letter or underscore"));
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(format!("{what} contains unsupported characters"));
    }
    Ok(())
}

pub fn program_module(spec: &ProgramSpec) -> String {
    let prefix = spec.option_prefix();
    format!(
        "{{ lib, config, ... }}:\n{{\n  options.{}.enable = lib.mkEnableOption \"{}\";\n\n  config = lib.mkIf config.{}.enable {{ }};\n}}\n",
        prefix,
        escape_nix_string(&spec.description),
        prefix
    )
}

pub fn capability_module(spec: &CapabilitySpec) -> String {
    let request_path = spec.request_path();
    format!(
        "{{ lib, config, ... }}:\nlet\n  requested = config.{} != [ ];\nin\n{{\n  config = lib.mkIf requested {{ }};\n}}\n",
        request_path
    )
}

pub fn profile_module(spec: &ProfileSpec) -> String {
    format!(
        "{{ lib, config, ... }}:\nlet\n  enabled = lib.elem \"{}\" config.gb.host.roles;\nin\n{{\n  config = lib.mkIf enabled {{\n    gb.debug.requests.enable = lib.mkDefault false;\n  }};\n}}\n",
        escape_nix_string(&spec.name)
    )
}

pub fn category_default(members: &[String]) -> String {
    let items = members
        .iter()
        .map(|m| format!("./{m}"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("{{\n  imports = [ {items} ];\n}}\n")
}

pub fn host_module(spec: &HostSpec) -> String {
    format!(
        "{{ config, lib, ... }}:\n{{\n  gb.host.name = \"{}\";\n  gb.host.system = \"{}\";\n}}\n",
        escape_nix_string(&spec.name),
        escape_nix_string(&spec.system)
    )
}

pub fn user_module(spec: &UserSpec) -> String {
    format!(
        "{{ config, lib, ... }}:\n{{\n  gb.user.username = \"{}\";\n  gb.user.fullName = \"{}\";\n  gb.user.email = \"{}\";\n}}\n",
        escape_nix_string(&spec.name),
        escape_nix_string(&spec.full_name),
        escape_nix_string(&spec.email)
    )
}

pub fn host_entry_body(spec: &HostSpec) -> String {
    format!(
        "  {} = {{\n    system = \"{}\";\n    class = \"desktop\";\n    roles = [ \"workstation\" ];\n    desktop = \"none\";\n    shell = \"zsh\";\n    hardware = {{\n      cpu = \"generic\";\n      gpu = \"none\";\n      hasBattery = false;\n      hasBluetooth = false;\n      hasTouchpad = false;\n      hasPrinter = false;\n    }};\n  }};",
        escape_nix_string(&spec.name),
        escape_nix_string(&spec.system)
    )
}

pub fn user_entry_body(spec: &UserSpec, is_main: bool) -> String {
    let name = escape_nix_string(&spec.name);
    format!(
        "  {name} = {{\n    username = \"{name}\";\n    fullName = \"{}\";\n    email = \"{}\";\n    isMain = {is_main};\n    homeModule = ./{name}.nix;\n  }};",
        escape_nix_string(&spec.full_name),
        escape_nix_string(&spec.email)
    )
}

pub fn append_entry(existing: &str, entry_body: &str) -> Result<String, String> {
    use avalanche_model::source::SourceSpan;
    use avalanche_source::{apply_patch, parse, SourcePatch};

    let file = parse(existing).map_err(|e| format!("enumeration file is not valid Nix: {e}"))?;
    let attrs = top_level_attrs(&file.expr).ok_or("enumeration file has no top-level attrset")?;
    let insert_at = last_binding_end(attrs).ok_or("enumeration attrset is empty")?;
    let patch = SourcePatch::new(
        "enumeration",
        SourceSpan::new(insert_at, insert_at),
        "",
        format!("\n{entry_body}"),
        "append entry",
    );
    apply_patch(existing, &patch).map_err(|e| format!("failed to append entry: {e}"))
}

fn last_binding_end(attrs: &[Binding]) -> Option<avalanche_model::source::Position> {
    attrs.last().map(|b| b.span.end)
}

fn top_level_attrs(expr: &Expr) -> Option<&[Binding]> {
    match expr {
        Expr::Attrs(bindings) => Some(bindings),
        Expr::Lambda { body, .. } => match body.as_ref() {
            Expr::Attrs(bindings) => Some(bindings),
            _ => None,
        },
        _ => None,
    }
}

pub fn init_skeleton(spec: &InitSpec) -> Vec<(String, String)> {
    let host_body = host_entry_body(&spec.host_spec());
    let user_body = user_entry_body(&spec.user_spec(), true);

    vec![
        ("flake.nix".to_string(), init_flake()),
        (
            "hosts/default.nix".to_string(),
            format!("{{\n{host_body}\n}}\n"),
        ),
        (
            format!("hosts/{}/default.nix", spec.host_name),
            host_module(&spec.host_spec()),
        ),
        (
            "users/default.nix".to_string(),
            format!("{{\n{user_body}\n}}\n"),
        ),
        (
            format!("users/{}.nix", spec.user_name),
            user_module(&spec.user_spec()),
        ),
        (
            "modules/aggregate/nixos.nix".to_string(),
            init_aggregate_nixos(),
        ),
        (
            "modules/aggregate/home.nix".to_string(),
            init_aggregate_home(),
        ),
        ("modules/core/schema.nix".to_string(), init_schema()),
        (
            "modules/core/host/options.nix".to_string(),
            init_host_options(),
        ),
        (
            "modules/core/user/options.nix".to_string(),
            init_user_options(),
        ),
        (
            "modules/core/requests/system.nix".to_string(),
            init_requests_system(),
        ),
        (
            "modules/core/requests/home.nix".to_string(),
            init_requests_home(),
        ),
        (
            "modules/core/debug/requests.nix".to_string(),
            init_debug_requests(),
        ),
    ]
}

fn scope_dir(scope: Scope) -> &'static str {
    match scope {
        Scope::System => "system",
        Scope::Home => "home",
    }
}

fn aggregate_path(scope: Scope) -> String {
    match scope {
        Scope::System => "modules/aggregate/nixos.nix".to_string(),
        Scope::Home => "modules/aggregate/home.nix".to_string(),
    }
}

fn domain_dir(domain: &str) -> String {
    format!("{}/", domain.replace('.', "/"))
}

fn relative_from_aggregate(path: &str) -> String {
    format!("../{}", path.strip_prefix("modules/").unwrap_or(path))
}

fn escape_nix_string(input: &str) -> String {
    let mut out = String::new();
    let mut it = input.char_indices().peekable();
    while let Some((_, c)) = it.next() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '$' => {
                if it.peek().map(|(_, n)| *n) == Some('{') {
                    out.push_str("\\${");
                    it.next();
                } else {
                    out.push('$');
                }
            }
            other => out.push(other),
        }
    }
    out
}

fn init_flake() -> String {
    r#"{
  description = "Avalanche repository";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager/release-24.11";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, home-manager, ... }: {
    nixosConfigurations = { };
    homeConfigurations = { };
  };
}
"#
    .to_string()
}

fn init_aggregate_nixos() -> String {
    r#"{ lib, config, ... }:
{
  imports = [
    ../core/schema.nix
    ../core/host/options.nix
    ../core/user/options.nix
    ../core/requests/system.nix
    ../core/debug/requests.nix
  ];

  config.networking.hostName = config.gb.host.name;
  config.nixpkgs.hostPlatform = lib.mkDefault config.gb.host.system;
  config.system.stateVersion = config.gb.host.stateVersion;
}
"#
    .to_string()
}

fn init_aggregate_home() -> String {
    r#"{ lib, config, ... }:
{
  imports = [
    ../core/schema.nix
    ../core/host/options.nix
    ../core/user/options.nix
    ../core/requests/home.nix
    ../core/debug/requests.nix
  ];

  config.home.stateVersion = config.gb.host.stateVersion;
  config.programs.home-manager.enable = true;
}
"#
    .to_string()
}

fn init_schema() -> String {
    r#"{ lib, ... }:
{
  options.gb.schemaVersion = lib.mkOption {
    type = lib.types.int;
    default = 1;
    readOnly = true;
    description = "Avalanche repository schema version";
  };

  config.gb.schemaVersion = 1;
}
"#
    .to_string()
}

fn init_host_options() -> String {
    r#"{ lib, ... }:
{
  options.gb.host = {
    name = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "Host name";
    };
    system = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "Host system";
    };
    class = lib.mkOption {
      type = lib.types.str;
      default = "desktop";
      description = "Host class";
    };
    roles = lib.mkOption {
      type = lib.types.listOf lib.types.str;
      default = [ ];
      description = "Host roles";
    };
    desktop = lib.mkOption {
      type = lib.types.str;
      default = "none";
      description = "Desktop compositor stack";
    };
    shell = lib.mkOption {
      type = lib.types.str;
      default = "zsh";
      description = "User shell";
    };
    stateVersion = lib.mkOption {
      type = lib.types.str;
      default = "24.11";
      description = "NixOS state version at install time";
    };
    hardware = {
      cpu = lib.mkOption {
        type = lib.types.str;
        default = "generic";
        description = "CPU vendor";
      };
      gpu = lib.mkOption {
        type = lib.types.str;
        default = "none";
        description = "GPU vendor";
      };
      hasBattery = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Host has battery";
      };
      hasBluetooth = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Host has bluetooth";
      };
      hasTouchpad = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Host has touchpad";
      };
      hasPrinter = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Host has printer";
      };
    };
  };
}
"#
    .to_string()
}

fn init_user_options() -> String {
    r#"{ lib, ... }:
{
  options.gb.user = {
    username = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "Primary username";
    };
    fullName = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "Primary user full name";
    };
    email = lib.mkOption {
      type = lib.types.str;
      default = "";
      description = "Primary user email";
    };
  };
}
"#
    .to_string()
}

fn init_requests_system() -> String {
    r#"{ lib, ... }:
{
  options.gb.requires.system = lib.mkOption {
    type = lib.types.attrsOf (lib.types.listOf lib.types.str);
    default = { };
    description = "System capability requests";
  };
}
"#
    .to_string()
}

fn init_requests_home() -> String {
    r#"{ lib, ... }:
{
  options.gb.requires.home = lib.mkOption {
    type = lib.types.attrsOf (lib.types.listOf lib.types.str);
    default = { };
    description = "Home capability requests";
  };
}
"#
    .to_string()
}

fn init_debug_requests() -> String {
    r#"{ lib, ... }:
{
  options.gb.debug.requests.enable = lib.mkEnableOption "Enable request debugging";
}
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_safe_components() {
        assert!(validate_component("pipewire", "x").is_ok());
        assert!(validate_component("..", "x").is_err());
        assert!(validate_component("a/b", "x").is_err());
        assert!(validate_dotted("dev.editors", "x").is_ok());
        assert!(validate_dotted("dev..editors", "x").is_err());
        assert!(validate_identifier("pc", "x").is_ok());
        assert!(validate_identifier("1pc", "x").is_err());
        assert!(validate_identifier("a.b", "x").is_err());
    }

    #[test]
    fn escapes_strings() {
        assert_eq!(escape_nix_string("a\"b"), "a\\\"b");
        assert_eq!(escape_nix_string("${x}"), "\\${x}");
        assert_eq!(escape_nix_string("a$b"), "a$b");
    }

    #[test]
    fn program_paths() {
        let sys = ProgramSpec {
            scope: Scope::System,
            category: String::new(),
            name: "steam".into(),
            description: "Steam".into(),
        };
        assert_eq!(sys.option_prefix(), "gb.programs.system.steam");
        assert_eq!(sys.module_path(), "modules/programs/system/steam.nix");
        assert_eq!(sys.relative_import(), "../programs/system/steam.nix");
        assert!(sys.category_default_path().is_none());

        let home = ProgramSpec {
            scope: Scope::Home,
            category: "web".into(),
            name: "firefox".into(),
            description: "Firefox".into(),
        };
        assert_eq!(home.option_prefix(), "gb.home.programs.web.firefox");
        assert_eq!(home.module_path(), "modules/programs/home/web/firefox.nix");
        assert_eq!(
            home.category_default_path().unwrap(),
            "modules/programs/home/web/default.nix"
        );
        assert_eq!(
            home.category_relative_import().unwrap(),
            "../programs/home/web/default.nix"
        );
    }

    #[test]
    fn capability_paths() {
        let cap = CapabilitySpec {
            scope: Scope::System,
            domain: "audio".into(),
            name: "pipewire".into(),
            description: "PipeWire".into(),
        };
        assert_eq!(cap.request_path(), "gb.requires.system.audio.pipewire");
        assert_eq!(
            cap.module_path(),
            "modules/capabilities/system/audio/pipewire.nix"
        );
        assert_eq!(
            cap.relative_import(),
            "../capabilities/system/audio/pipewire.nix"
        );
    }

    #[test]
    fn profile_paths() {
        let prof = ProfileSpec {
            scope: Scope::System,
            name: "workstation".into(),
        };
        assert_eq!(prof.requester(), "profiles.system.workstation");
        assert_eq!(
            prof.module_path(),
            "modules/profiles/system/workstation.nix"
        );
    }

    #[test]
    fn append_entry_to_attrset() {
        let existing = "{\n  pc = {\n    system = \"x86_64-linux\";\n  };\n}\n";
        let entry = "  laptop = {\n    system = \"x86_64-linux\";\n  };";
        let result = append_entry(existing, entry).unwrap();
        assert!(result.contains("laptop"));
        assert!(result.contains("pc"));
        avalanche_source::parse(&result).unwrap();
    }
}
