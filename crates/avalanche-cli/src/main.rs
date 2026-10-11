use std::process;

use avalanche_core::{ConfigurationEngine, RepositoryIndex};
use avalanche_model::{Module, ModuleKind, OptionDefinition, Repository};
use serde::Serialize;
use serde_json::{json, Value};

fn main() {
    process::exit(run());
}

struct Args {
    repo: String,
    json: bool,
    use_nix: bool,
    help: bool,
    command: String,
    rest: Vec<String>,
}

struct CmdOut {
    value: Value,
    text: String,
    code: i32,
}

fn ok(value: Value, text: String) -> CmdOut {
    CmdOut {
        value,
        text,
        code: 0,
    }
}

fn run() -> i32 {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&argv) {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("avalanche: {}", msg);
            eprint_usage();
            return 2;
        }
    };

    if args.help || args.command.is_empty() {
        print_help();
        return 0;
    }

    let repo_model = Repository::new(&args.repo);
    let engine = ConfigurationEngine::new(&args.repo);

    let needs_index = !matches!(args.command.as_str(), "diff" | "init");
    let built = if needs_index {
        let res = if args.use_nix {
            engine.index(&repo_model)
        } else {
            engine.index_static(&repo_model)
        };
        match res {
            Ok(i) => Some(i),
            Err(e) => {
                eprintln!("failed to index repository '{}': {}", args.repo, e);
                return 1;
            }
        }
    } else {
        None
    };

    let out = dispatch(&engine, built.as_ref(), &args);
    match out {
        Ok(o) => {
            emit(args.json, &o.value, &o.text);
            o.code
        }
        Err(msg) => {
            eprintln!("avalanche {}: {}", args.command, msg);
            1
        }
    }
}

fn dispatch(
    engine: &ConfigurationEngine,
    index: Option<&RepositoryIndex>,
    args: &Args,
) -> Result<CmdOut, String> {
    match args.command.as_str() {
        "inspect" => cmd_inspect(need_index(index)?, args),
        "why" => cmd_why(engine, need_index(index)?, args),
        "dependents" => cmd_dependents(engine, need_index(index)?, args),
        "owners" => cmd_owners(engine, need_index(index)?, args),
        "doctor" => cmd_doctor(engine, need_index(index)?),
        "diff" => cmd_diff(engine),
        "host" => cmd_host(engine, need_index(index)?, args),
        "user" => cmd_user(engine, need_index(index)?, args),
        "generate" => cmd_generate(engine, need_index(index)?, args),
        "init" => cmd_init(args),
        other => Err(format!("unknown command '{}'", other)),
    }
}

fn need_index(index: Option<&RepositoryIndex>) -> Result<&RepositoryIndex, String> {
    index.ok_or_else(|| "command requires an index".to_string())
}

fn cmd_inspect(idx: &RepositoryIndex, args: &Args) -> Result<CmdOut, String> {
    let kind = args
        .rest
        .first()
        .ok_or("usage: inspect <host|user|profile|option|module|capability|program|desktop|aggregate|core> <name>")?
        .clone();
    let name = args.rest.get(1).ok_or("inspect requires a name")?.clone();

    match kind.as_str() {
        "host" => {
            let h = idx
                .hosts
                .iter()
                .find(|h| h.name == name)
                .ok_or_else(|| format!("unknown host '{}'", name))?;
            let text = format!(
                "host {}\n  system: {}\n  class: {:?}\n  roles: {:?}\n  desktop: {:?}\n  shell: {:?}\n  stateVersion: {}\n  hardware: {:?}\n",
                h.name, h.system, h.class, h.roles, h.desktop, h.shell, h.state_version, h.hardware
            );
            Ok(ok(jv(h), text))
        }
        "user" => {
            let u = idx
                .users
                .iter()
                .find(|u| u.username == name)
                .ok_or_else(|| format!("unknown user '{}'", name))?;
            let text = format!(
                "user {}\n  fullName: {}\n  email: {}\n  main: {}\n",
                u.username, u.full_name, u.email, u.is_main
            );
            Ok(ok(jv(u), text))
        }
        "profile" => {
            let p = idx
                .profiles
                .iter()
                .find(|p| p.id.name == name)
                .ok_or_else(|| format!("unknown profile '{}'", name))?;
            let text = format!(
                "profile {}\n  scope: {:?}\n  source: {}\n",
                p.id.name, p.id.scope, p.source_file
            );
            Ok(ok(jv(p), text))
        }
        "option" => {
            let prov = idx.provenance_for(&name);
            let schema = idx.option(&name);
            if prov.is_none() && schema.is_none() {
                return Err(format!("unknown option '{}'", name));
            }
            let mut map = serde_json::Map::new();
            map.insert("path".to_string(), json!(name));
            if let Some(s) = schema {
                map.insert("schema".to_string(), jv(s));
            }
            if let Some(p) = prov {
                map.insert("provenance".to_string(), jv(p));
            }
            let mut text = format!("option {}\n", name);
            if let Some(s) = schema {
                if let Some(d) = &s.description {
                    text.push_str(&format!("  description: {}\n", d));
                }
                text.push_str(&format!("  type: {:?}\n", s.nix_type));
            }
            match prov {
                Some(p) => {
                    if let Some(w) = p.winning_definition() {
                        text.push_str(&format!("  effective via: {}\n", def_line(w)));
                    }
                    for d in &p.definitions {
                        text.push_str(&format!("  def: {}\n", def_line(d)));
                    }
                }
                None => text.push_str("  (no provenance)\n"),
            }
            Ok(ok(Value::Object(map), text))
        }
        "module" | "capability" | "program" | "desktop" | "aggregate" | "core" => {
            let wk = parse_module_kind(&kind);
            let mods: Vec<&Module> = idx
                .modules
                .iter()
                .filter(|m| {
                    let kind_ok = match &wk {
                        Some(k) => &m.id.kind == k,
                        None => true,
                    };
                    kind_ok && module_matches(m, &name)
                })
                .collect();
            if mods.is_empty() {
                return Err(format!("no {} matching '{}'", kind, name));
            }
            let mut arr = Vec::new();
            let mut text = String::new();
            for m in &mods {
                let opts = options_in(idx, &m.path);
                arr.push(json!({
                    "path": m.path,
                    "kind": format!("{:?}", m.id.kind),
                    "name": m.id.name,
                    "imported": m.imported,
                    "options": opts,
                }));
                text.push_str(&format!("{} [{:?}] {}\n", m.path, m.id.kind, m.id.name));
                text.push_str(&format!("  imported: {}\n", m.imported));
                text.push_str(&format!("  options ({}):\n", opts.len()));
                for o in &opts {
                    text.push_str(&format!("    {}\n", o));
                }
            }
            Ok(ok(Value::Array(arr), text))
        }
        other => Err(format!("unknown inspect kind '{}'", other)),
    }
}

fn cmd_why(
    engine: &ConfigurationEngine,
    idx: &RepositoryIndex,
    args: &Args,
) -> Result<CmdOut, String> {
    let input = args
        .rest
        .first()
        .ok_or("usage: why <capability|option>")?
        .clone();
    let chains = engine.why(idx, &input)?;
    let mut text = String::new();
    if chains.is_empty() {
        text.push_str(&format!("why {}: (no requesters)\n", input));
    } else {
        for c in &chains {
            text.push_str(&format!("via {}:\n", c.via));
            text.push_str(&format!("  {}\n", c.steps.join(" -> ")));
        }
    }
    Ok(ok(jv(&chains), text))
}

fn cmd_dependents(
    engine: &ConfigurationEngine,
    idx: &RepositoryIndex,
    args: &Args,
) -> Result<CmdOut, String> {
    let input = args
        .rest
        .first()
        .ok_or("usage: dependents <profile|capability>")?
        .clone();
    let deps = engine.dependents(idx, &input)?;
    let mut text = String::new();
    if deps.is_empty() {
        text.push_str(&format!("dependents {}: (none)\n", input));
    } else {
        for d in &deps {
            text.push_str(&format!("  {}\n", d));
        }
    }
    Ok(ok(jv(&deps), text))
}

fn cmd_owners(
    engine: &ConfigurationEngine,
    idx: &RepositoryIndex,
    args: &Args,
) -> Result<CmdOut, String> {
    let path = args
        .rest
        .first()
        .ok_or("usage: owners <option-path>")?
        .clone();
    let entry = engine.owners(idx, &path)?;
    let text = format!(
        "option {}\n  owner: {} ({})\n  status: {:?}\n  files: {:?}\n",
        entry.option_path,
        entry.owner_module,
        entry.owner_kind,
        entry.status,
        entry.contributing_files
    );
    Ok(ok(jv(&entry), text))
}

fn cmd_doctor(engine: &ConfigurationEngine, idx: &RepositoryIndex) -> Result<CmdOut, String> {
    let diag = engine.diagnose(idx);
    let mut text = format!(
        "doctor: {} errors, {} warnings\n",
        diag.error_count(),
        diag.warning_count()
    );
    for f in &diag.findings {
        let loc = f.path.as_deref().or(f.file.as_deref()).unwrap_or("-");
        text.push_str(&format!("  [{:?}] {}: {}\n", f.severity, loc, f.message));
    }
    if !diag.repairs.is_empty() {
        text.push_str("repairs:\n");
        for r in &diag.repairs {
            text.push_str(&format!(
                "  ({:.0}%) {}\n",
                r.confidence * 100.0,
                r.description
            ));
        }
    }
    let code = if diag.has_errors() { 1 } else { 0 };
    Ok(CmdOut {
        value: jv(&diag),
        text,
        code,
    })
}

fn cmd_host(
    engine: &ConfigurationEngine,
    idx: &RepositoryIndex,
    args: &Args,
) -> Result<CmdOut, String> {
    let sub = args
        .rest
        .first()
        .ok_or("usage: host <create> <name>")?
        .as_str();
    match sub {
        "create" => {
            let name = args
                .rest
                .get(1)
                .ok_or("host create requires a name")?
                .clone();
            let intent = avalanche_model::MutationIntent::CreateHost { name: name.clone() };
            let txn = engine
                .execute_mutation(idx, vec![intent])
                .map_err(|e| e.to_string())?;
            let diff = txn.diff.clone().unwrap_or_default();
            let text = format!("created host '{}'\n{}", name, diff);
            Ok(ok(jv(&txn), text))
        }
        other => Err(format!("unknown host subcommand '{}'", other)),
    }
}

fn cmd_user(
    engine: &ConfigurationEngine,
    idx: &RepositoryIndex,
    args: &Args,
) -> Result<CmdOut, String> {
    let sub = args
        .rest
        .first()
        .ok_or("usage: user <create> <name>")?
        .as_str();
    match sub {
        "create" => {
            let name = args
                .rest
                .get(1)
                .ok_or("user create requires a name")?
                .clone();
            let intent = avalanche_model::MutationIntent::CreateUser { name: name.clone() };
            let txn = engine
                .execute_mutation(idx, vec![intent])
                .map_err(|e| e.to_string())?;
            let diff = txn.diff.clone().unwrap_or_default();
            let text = format!("created user '{}'\n{}", name, diff);
            Ok(ok(jv(&txn), text))
        }
        other => Err(format!("unknown user subcommand '{}'", other)),
    }
}

fn cmd_generate(
    engine: &ConfigurationEngine,
    idx: &RepositoryIndex,
    args: &Args,
) -> Result<CmdOut, String> {
    let kind = args
        .rest
        .first()
        .ok_or("usage: generate <program|capability|profile> <name>")?
        .clone();
    let name = args.rest.get(1).ok_or("generate requires a name")?.clone();
    let intent = avalanche_model::MutationIntent::CreateModule { name, kind };
    let txn = engine
        .execute_mutation(idx, vec![intent])
        .map_err(|e| e.to_string())?;
    let diff = txn.diff.clone().unwrap_or_default();
    Ok(ok(jv(&txn), diff))
}

fn cmd_init(args: &Args) -> Result<CmdOut, String> {
    let mut host = String::new();
    let mut user = String::new();
    let mut full_name = String::new();
    let mut email = String::new();
    let mut system = "x86_64-linux".to_string();
    let mut target = String::new();
    let rest = &args.rest;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--host" => {
                i += 1;
                host = rest.get(i).ok_or("--host requires a value")?.clone();
            }
            "--user" => {
                i += 1;
                user = rest.get(i).ok_or("--user requires a value")?.clone();
            }
            "--full-name" => {
                i += 1;
                full_name = rest.get(i).ok_or("--full-name requires a value")?.clone();
            }
            "--email" => {
                i += 1;
                email = rest.get(i).ok_or("--email requires a value")?.clone();
            }
            "--system" => {
                i += 1;
                system = rest.get(i).ok_or("--system requires a value")?.clone();
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown init flag '{other}'"));
            }
            other => {
                if target.is_empty() {
                    target = other.to_string();
                } else {
                    return Err(format!("unexpected argument '{other}'"));
                }
            }
        }
        i += 1;
    }
    if target.is_empty() {
        return Err("usage: init <path> --host <h> --user <u>".to_string());
    }
    let spec = avalanche_write::InitSpec {
        host_name: host,
        host_system: system,
        user_name: user,
        user_full_name: full_name,
        user_email: email,
    };
    avalanche_core::ConfigurationEngine::init_repository(std::path::Path::new(&target), &spec)
        .map_err(|e| e.to_string())?;
    Ok(ok(
        json!({"initialized": target}),
        format!("initialized repository at {target}\n"),
    ))
}

fn cmd_diff(engine: &ConfigurationEngine) -> Result<CmdOut, String> {
    let d = engine.diff()?;
    let text = if d.is_empty() {
        "(no changes)\n".to_string()
    } else {
        d.clone()
    };
    Ok(ok(json!({ "diff": d }), text))
}

fn parse_args(argv: &[String]) -> Result<Args, String> {
    let mut repo = ".".to_string();
    let mut json = false;
    let mut use_nix = false;
    let mut help = false;
    let mut command = String::new();
    let mut rest: Vec<String> = Vec::new();
    let mut i = 0;
    while i < argv.len() {
        let tok = argv[i].as_str();
        match tok {
            "--repo" | "-r" => {
                i += 1;
                repo = argv
                    .get(i)
                    .ok_or_else(|| "--repo requires a value".to_string())?
                    .clone();
            }
            "--json" => json = true,
            "--nix" | "--eval" => use_nix = true,
            "-h" | "--help" => help = true,
            other if other.starts_with('-') => {
                return Err(format!("unknown flag '{}'", other));
            }
            other => {
                if command.is_empty() {
                    command = other.to_string();
                } else {
                    rest.push(other.to_string());
                }
            }
        }
        i += 1;
    }
    Ok(Args {
        repo,
        json,
        use_nix,
        help,
        command,
        rest,
    })
}

fn emit(json: bool, value: &Value, text: &str) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
        );
    } else {
        print!("{}", text);
    }
}

fn jv<T: Serialize>(t: &T) -> Value {
    serde_json::to_value(t).unwrap_or(Value::Null)
}

fn def_line(d: &OptionDefinition) -> String {
    let value = d
        .value
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or_else(|| "n/a".to_string());
    format!(
        "{}:{} layer={:?} type={:?} value={}",
        d.source.file, d.source.span.start.line, d.layer, d.definition_type, value
    )
}

fn module_matches(m: &Module, query: &str) -> bool {
    let path = m.path.replace('\\', "/");
    let qslash = query.replace('.', "/");
    let base = path
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim_end_matches(".nix");
    m.id.name == query || path.contains(&qslash) || path.contains(query) || base.contains(query)
}

fn options_in(idx: &RepositoryIndex, module_path: &str) -> Vec<String> {
    let mut v: Vec<String> = idx
        .provenance
        .iter()
        .filter(|(_, p)| p.definitions.iter().any(|d| d.source.file == module_path))
        .map(|(k, _)| k.clone())
        .collect();
    v.sort();
    v
}

fn parse_module_kind(s: &str) -> Option<ModuleKind> {
    match s {
        "capability" => Some(ModuleKind::Capability),
        "program" => Some(ModuleKind::Program),
        "desktop" => Some(ModuleKind::Desktop),
        "aggregate" => Some(ModuleKind::Aggregate),
        "core" => Some(ModuleKind::Core),
        _ => None,
    }
}

fn print_help() {
    println!("avalanche — Nix configuration manager");
    println!();
    println!("USAGE:");
    println!("  avalanche [OPTIONS] <COMMAND>");
    println!();
    println!("OPTIONS:");
    println!("  -r, --repo <PATH>   repository path (default: .)");
    println!("      --json          emit JSON");
    println!("      --nix           use full Nix evaluation index (default: static)");
    println!("  -h, --help          print help");
    println!();
    println!("COMMANDS:");
    println!("  inspect <kind> <name>   host|user|profile|option|module|capability|program|desktop|aggregate|core");
    println!("  why <capability|option> explain why an effective value exists");
    println!("  dependents <node>       what a requester pulls in");
    println!("  owners <option-path>    who owns an upstream option");
    println!("  doctor                  repository diagnostics + repair proposals");
    println!("  diff                    working-tree diff");
    println!("  host create <name>      create a new host");
    println!("  user create <name>      create a new user");
    println!("  generate <kind> <name>  generate a module (program|capability|profile)");
    println!("  init <path> --host <h> --user <u>  initialize a new repository");
}

fn eprint_usage() {
    eprintln!();
    eprintln!("run 'avalanche --help' for usage");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(s: &str) -> String {
        s.to_string()
    }

    #[test]
    fn parses_repo_flag_and_command() {
        let args = parse_args(&[a("-r"), a("/tmp/x"), a("why"), a("audio.pipewire")]).unwrap();
        assert_eq!(args.repo, "/tmp/x");
        assert_eq!(args.command, "why");
        assert_eq!(args.rest, vec![a("audio.pipewire")]);
        assert!(!args.json);
        assert!(!args.use_nix);
    }

    #[test]
    fn parses_json_and_nix_flags() {
        let args = parse_args(&[a("--json"), a("--nix"), a("doctor")]).unwrap();
        assert!(args.json);
        assert!(args.use_nix);
        assert_eq!(args.command, "doctor");
    }

    #[test]
    fn unknown_flag_is_error() {
        assert!(parse_args(&[a("--bogus")]).is_err());
    }

    #[test]
    fn repo_flag_without_value_is_error() {
        assert!(parse_args(&[a("--repo")]).is_err());
    }

    #[test]
    fn help_flag_set() {
        let args = parse_args(&[a("--help")]).unwrap();
        assert!(args.help);
    }

    #[test]
    fn module_matches_by_name_and_path() {
        let m = Module {
            id: avalanche_model::ModuleId {
                kind: ModuleKind::Capability,
                scope: Some(avalanche_model::Scope::System),
                name: "audio.pipewire".to_string(),
            },
            path: "modules/capabilities/system/audio/pipewire.nix".to_string(),
            imported: true,
        };
        assert!(module_matches(&m, "audio.pipewire"));
        assert!(module_matches(&m, "pipewire"));
        assert!(module_matches(&m, "capabilities/system/audio"));
        assert!(!module_matches(&m, "zzz"));
    }

    #[test]
    fn parse_module_kind_maps() {
        assert_eq!(
            parse_module_kind("capability"),
            Some(ModuleKind::Capability)
        );
        assert_eq!(parse_module_kind("module"), None);
        assert_eq!(parse_module_kind("nonsense"), None);
    }

    #[test]
    fn def_line_renders() {
        use avalanche_model::source::{Position, SourceLocation, SourceSpan};
        use avalanche_model::{DefinitionType, Layer, Priority, Scope};
        let d = OptionDefinition {
            option_path: "services.pipewire.enable".to_string(),
            value: Some(json!(true)),
            source: SourceLocation::new(
                "modules/capabilities/system/audio/pipewire.nix",
                SourceSpan::new(
                    Position { line: 8, column: 1 },
                    Position { line: 8, column: 9 },
                ),
            ),
            priority: Priority::normal(),
            layer: Layer::Capability,
            scope: Scope::System,
            host: None,
            user: None,
            profile: None,
            condition: None,
            definition_type: DefinitionType::Literal,
        };
        let line = def_line(&d);
        assert!(line.contains("pipewire.nix:8"));
        assert!(line.contains("value=true"));
    }
}
