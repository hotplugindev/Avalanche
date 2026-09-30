use std::collections::HashMap;

use avalanche_model::option::{EffectiveValue, OptionSchema};
use avalanche_model::request::ResolvedRequest;
use avalanche_model::scope::Scope;
use avalanche_nix::{Assertion, NixService};
use serde_json::Value;

use crate::error::{IndexError, IndexResult};

#[derive(Debug, Clone)]
pub struct SemanticIndex {
    pub hosts: Vec<String>,
    pub effective: HashMap<String, EffectiveValue>,
    pub schemas: HashMap<String, OptionSchema>,
    pub assertions: HashMap<String, Vec<Assertion>>,
    pub requests: Vec<ResolvedRequest>,
    pub flake_inputs: Vec<avalanche_nix::FlakeInput>,
    pub eval_errors: Vec<EvalErrorRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalErrorRecord {
    pub host: String,
    pub reason: String,
}

impl SemanticIndex {
    pub fn empty() -> Self {
        Self {
            hosts: Vec::new(),
            effective: HashMap::new(),
            schemas: HashMap::new(),
            assertions: HashMap::new(),
            requests: Vec::new(),
            flake_inputs: Vec::new(),
            eval_errors: Vec::new(),
        }
    }

    pub fn effective_for(&self, path: &str) -> Option<&EffectiveValue> {
        self.effective.get(path)
    }

    pub fn schema_for(&self, path: &str) -> Option<&OptionSchema> {
        self.schemas.get(path)
    }

    pub fn assertions_for_host(&self, host: &str) -> &[Assertion] {
        self.assertions
            .get(host)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn failed_assertions(&self) -> Vec<(&str, &Assertion)> {
        self.assertions
            .iter()
            .flat_map(|(host, asserts)| {
                asserts
                    .iter()
                    .filter(|a| !a.passed)
                    .map(move |a| (host.as_str(), a))
            })
            .collect()
    }

    pub fn active_requests(&self) -> Vec<&ResolvedRequest> {
        self.requests.iter().filter(|r| r.active).collect()
    }

    pub fn has_eval_errors(&self) -> bool {
        !self.eval_errors.is_empty()
    }
}

pub fn build_semantic_index(nix: &NixService) -> IndexResult<SemanticIndex> {
    let mut index = SemanticIndex::empty();

    let hosts = nix.list_hosts().map_err(|e| IndexError::EvalFailed {
        reason: format!("listing hosts: {}", e),
    })?;
    index.hosts = hosts.clone();

    for host in &hosts {
        match nix.eval_host(host) {
            Ok(host_result) => extract_gb_values(&host_result.config, &mut index),
            Err(e) => index.eval_errors.push(EvalErrorRecord {
                host: host.clone(),
                reason: e.to_string(),
            }),
        }

        match nix.get_assertions(host) {
            Ok(assertions) => {
                index.assertions.insert(host.clone(), assertions);
            }
            Err(e) => index.eval_errors.push(EvalErrorRecord {
                host: host.clone(),
                reason: format!("assertions: {}", e),
            }),
        }
    }

    index.requests = extract_requests(&index);

    if let Ok(inputs) = nix.discover_inputs() {
        index.flake_inputs = inputs;
    }

    Ok(index)
}

fn extract_gb_values(config: &Value, index: &mut SemanticIndex) {
    if let Some(gb) = config.get("gb") {
        walk_json_attrs(gb, &[], &mut index.effective);
    }
}

fn walk_json_attrs(value: &Value, prefix: &[String], out: &mut HashMap<String, EffectiveValue>) {
    if let Value::Object(map) = value {
        for (key, val) in map {
            let mut path = prefix.to_vec();
            path.push(key.clone());
            match val {
                Value::Object(_) => walk_json_attrs(val, &path, out),
                leaf => {
                    let dot_path = format!("gb.{}", path.join("."));
                    out.insert(
                        dot_path.clone(),
                        EffectiveValue {
                            option_path: dot_path,
                            value: Some(avalanche_model::JsonValue(leaf.clone())),
                            definitions: Vec::new(),
                        },
                    );
                }
            }
        }
    }
}

fn extract_requests(index: &SemanticIndex) -> Vec<ResolvedRequest> {
    let mut requests = Vec::new();

    for (path, effective) in &index.effective {
        if !path.starts_with("gb.requires.") {
            continue;
        }

        let requesters: Vec<String> = match &effective.value {
            Some(avalanche_model::JsonValue(Value::Array(arr))) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            _ => Vec::new(),
        };

        let cap_path = path.strip_prefix("gb.requires.").unwrap_or(path);
        let parts: Vec<&str> = cap_path.splitn(3, '.').collect();
        if parts.len() < 2 {
            continue;
        }

        let scope = match parts[0] {
            "system" => Scope::System,
            "home" => Scope::Home,
            _ => continue,
        };

        let name_parts: Vec<&str> = parts[1..].iter().copied().collect();
        let (domain, name) = if name_parts.len() >= 2 {
            (
                name_parts[..name_parts.len() - 1].join("."),
                name_parts[name_parts.len() - 1].to_string(),
            )
        } else {
            (String::new(), name_parts.join("."))
        };

        let active = !effective
            .value
            .as_ref()
            .map(|v| match &v.0 {
                Value::Array(arr) => arr.is_empty(),
                _ => true,
            })
            .unwrap_or(true);

        requests.push(ResolvedRequest {
            capability: avalanche_model::CapabilityId {
                scope,
                domain,
                name,
            },
            requesters,
            active,
        });
    }

    requests.sort_by(|a, b| a.capability.request_path().cmp(&b.capability.request_path()));
    requests
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_requests_from_effective() {
        let mut index = SemanticIndex::empty();
        index.effective.insert(
            "gb.requires.system.audio.pipewire".to_string(),
            EffectiveValue {
                option_path: "gb.requires.system.audio.pipewire".to_string(),
                value: Some(avalanche_model::JsonValue(serde_json::json!([
                    "programs.system.steam"
                ]))),
                definitions: Vec::new(),
            },
        );
        index.effective.insert(
            "gb.requires.home.git".to_string(),
            EffectiveValue {
                option_path: "gb.requires.home.git".to_string(),
                value: Some(avalanche_model::JsonValue(serde_json::json!([]))),
                definitions: Vec::new(),
            },
        );

        let requests = extract_requests(&index);
        assert_eq!(requests.len(), 2);

        let pipewire = requests
            .iter()
            .find(|r| r.capability.name == "pipewire")
            .unwrap();
        assert!(pipewire.active);
        assert_eq!(pipewire.capability.domain, "audio");
        assert_eq!(pipewire.capability.scope, Scope::System);
        assert_eq!(pipewire.requesters, vec!["programs.system.steam"]);

        let git = requests.iter().find(|r| r.capability.name == "git").unwrap();
        assert!(!git.active);
        assert_eq!(git.capability.scope, Scope::Home);
    }

    #[test]
    fn walk_json_attrs_nested() {
        let mut out = HashMap::new();
        let gb = serde_json::json!({
            "schemaVersion": 1,
            "programs": {
                "git": { "enable": true }
            },
            "requires": {
                "system": { "audio": { "pipewire": ["programs.system.steam"] } }
            }
        });
        walk_json_attrs(&gb, &[], &mut out);
        assert!(out.contains_key("gb.schemaVersion"));
        assert!(out.contains_key("gb.programs.git.enable"));
        assert!(out.contains_key("gb.requires.system.audio.pipewire"));
        match out["gb.programs.git.enable"].value {
            Some(avalanche_model::JsonValue(Value::Bool(true))) => {}
            ref other => panic!("expected bool true, got {:?}", other),
        }
    }

    #[test]
    fn failed_assertions_filter() {
        let mut index = SemanticIndex::empty();
        index.assertions.insert(
            "pc".to_string(),
            vec![
                Assertion {
                    message: "ok".into(),
                    passed: true,
                },
                Assertion {
                    message: "bad".into(),
                    passed: false,
                },
            ],
        );
        let failed = index.failed_assertions();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].0, "pc");
        assert_eq!(failed[0].1.message, "bad");
    }

    #[test]
    fn semantic_index_empty() {
        let index = SemanticIndex::empty();
        assert!(index.hosts.is_empty());
        assert!(index.effective.is_empty());
        assert!(index.failed_assertions().is_empty());
        assert!(!index.has_eval_errors());
    }

    #[test]
    fn extract_gb_values_ignores_non_gb_config() {
        let mut index = SemanticIndex::empty();
        let config = serde_json::json!({ "services": { "pipewire": { "enable": true } } });
        extract_gb_values(&config, &mut index);
        assert!(index.effective.is_empty());
    }
}
