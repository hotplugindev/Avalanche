use std::collections::{BTreeMap, HashMap};

use avalanche_model::{source::SourceLocation, Layer, OptionDefinition, Provenance, Scope};
use avalanche_source::analyze_definition;

use crate::semantic_index::SemanticIndex;
use crate::static_index::StaticIndex;

pub fn build_provenance_index(
    static_idx: &StaticIndex,
    semantic_idx: &SemanticIndex,
) -> HashMap<String, Provenance> {
    let mut provenance: BTreeMap<String, Provenance> = BTreeMap::new();

    for file in &static_idx.parsed_files {
        let context = context_for_file(&file.relative_path);

        for assignment in &file.assignments {
            let option_path = assignment.dot_path();
            if !is_configuration_path(&option_path) {
                continue;
            }

            let info = analyze_definition(&assignment.value);
            let definition = OptionDefinition {
                option_path: option_path.clone(),
                value: info.value.map(|v| v.0),
                source: SourceLocation::new(file.relative_path.clone(), assignment.span),
                priority: info.priority,
                layer: context.layer.clone(),
                scope: infer_scope(&option_path, context.scope),
                host: context.host.clone(),
                user: context.user.clone(),
                profile: context.profile.clone(),
                condition: info.condition,
                definition_type: info.definition_type,
            };

            provenance
                .entry(option_path.clone())
                .or_insert_with(|| Provenance::new(option_path.clone()))
                .add(definition);
        }
    }

    for (option_path, effective) in &semantic_idx.effective {
        provenance
            .entry(option_path.clone())
            .or_insert_with(|| Provenance::new(option_path.clone()))
            .set_effective_value(effective.value.clone());
    }

    for prov in provenance.values_mut() {
        prov.sort_definitions();
    }

    provenance.into_iter().collect()
}

#[derive(Debug, Clone)]
struct FileContext {
    layer: Layer,
    scope: Option<Scope>,
    host: Option<String>,
    user: Option<String>,
    profile: Option<String>,
}

fn context_for_file(path: &str) -> FileContext {
    let path = path.replace('\\', "/");

    if let Some(rest) = path.strip_prefix("hosts/") {
        let mut parts = rest.split('/');
        let host = parts.next().map(str::to_string);
        let remainder: Vec<&str> = parts.collect();
        if remainder.first().copied() == Some("home") {
            let user = remainder.get(1).copied().map(trim_nix);
            return FileContext {
                layer: Layer::User,
                scope: Some(Scope::Home),
                host,
                user,
                profile: None,
            };
        }
        return FileContext {
            layer: Layer::Host,
            scope: Some(Scope::System),
            host,
            user: None,
            profile: None,
        };
    }

    if let Some(rest) = path.strip_prefix("users/") {
        let user = rest.split('/').next().map(trim_nix);
        return FileContext {
            layer: Layer::User,
            scope: Some(Scope::Home),
            host: None,
            user,
            profile: None,
        };
    }

    if let Some(rest) = path.strip_prefix("modules/") {
        let kind = rest.split('/').next().unwrap_or("");
        let scope = scope_in_path(&path);
        match kind {
            "profiles" => {
                let profile = module_name_after_kind(rest, "profiles");
                return FileContext {
                    layer: Layer::Profile,
                    scope,
                    host: None,
                    user: None,
                    profile,
                };
            }
            "capabilities" => {
                return FileContext {
                    layer: Layer::Capability,
                    scope,
                    host: None,
                    user: None,
                    profile: None,
                };
            }
            "programs" => {
                return FileContext {
                    layer: Layer::Program,
                    scope,
                    host: None,
                    user: None,
                    profile: None,
                };
            }
            "desktop" => {
                return FileContext {
                    layer: Layer::Desktop,
                    scope,
                    host: None,
                    user: None,
                    profile: None,
                };
            }
            _ => {}
        }
    }

    FileContext {
        layer: Layer::Host,
        scope: scope_in_path(&path),
        host: None,
        user: None,
        profile: None,
    }
}

fn scope_in_path(path: &str) -> Option<Scope> {
    if path.contains("/system/") || path.starts_with("system/") {
        Some(Scope::System)
    } else if path.contains("/home/") || path.starts_with("home/") {
        Some(Scope::Home)
    } else {
        None
    }
}

fn module_name_after_kind(rest: &str, kind: &str) -> Option<String> {
    let after_kind = rest.strip_prefix(kind)?.strip_prefix('/')?;
    let without_scope = after_kind
        .strip_prefix("system/")
        .or_else(|| after_kind.strip_prefix("home/"))
        .unwrap_or(after_kind);
    let name = without_scope
        .trim_end_matches(".nix")
        .trim_end_matches("/default");
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn trim_nix(value: &str) -> String {
    value.trim_end_matches(".nix").to_string()
}

fn infer_scope(option_path: &str, file_scope: Option<Scope>) -> Scope {
    if let Some(scope) = file_scope {
        return scope;
    }
    if option_path.starts_with("gb.home.")
        || option_path.starts_with("gb.requires.home.")
        || option_path.contains(".home.")
    {
        Scope::Home
    } else {
        Scope::System
    }
}

fn is_configuration_path(option_path: &str) -> bool {
    option_path != "imports"
        && !option_path.starts_with("imports.")
        && option_path != "key"
        && option_path != "_module"
        && !option_path.starts_with("_module.")
        && option_path != "options"
        && !option_path.starts_with("options.")
        && option_path != "meta"
        && !option_path.starts_with("meta.")
}
