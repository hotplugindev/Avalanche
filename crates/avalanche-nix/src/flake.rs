use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{NixError, NixResult};
use crate::process::NixProcess;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlakeInput {
    pub name: String,
    pub url: String,
    pub rev: Option<String>,
    pub is_flake: bool,
}

pub fn discover_inputs(process: &NixProcess, repo_path: &Path) -> NixResult<Vec<FlakeInput>> {
    let args = ["flake", "metadata", "--json"];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::FlakeDiscoveryFailed {
            reason: output.stderr,
        });
    }

    let json: Value = serde_json::from_str(&output.stdout).map_err(|e| {
        NixError::FlakeDiscoveryFailed {
            reason: e.to_string(),
        }
    })?;

    parse_flake_inputs(&json)
}

pub fn parse_flake_inputs(metadata: &Value) -> NixResult<Vec<FlakeInput>> {
    let locks = match metadata.get("locks").or_else(|| metadata.get("lock")) {
        Some(l) => l,
        None => return Ok(Vec::new()),
    };

    let nodes = match locks.get("nodes") {
        Some(n) => n,
        None => return Ok(Vec::new()),
    };

    let root = locks
        .get("root")
        .and_then(|r| r.as_str())
        .unwrap_or("root");

    let root_node = match nodes.get(root) {
        Some(n) => n,
        None => return Ok(Vec::new()),
    };

    let inputs = match root_node.get("inputs").and_then(|i| i.as_object()) {
        Some(i) => i,
        None => return Ok(Vec::new()),
    };

    let mut result = Vec::new();

    for (name, input_ref) in inputs {
        let node_name = match input_ref.as_str() {
            Some(n) => n.to_string(),
            None => continue,
        };

        let node = match nodes.get(&node_name) {
            Some(n) => n,
            None => continue,
        };

        let locked = node.get("locked");
        let original = node.get("original");

        let url = original
            .and_then(|o| o.get("url"))
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();

        let rev = locked
            .and_then(|l| l.get("rev"))
            .and_then(|r| r.as_str())
            .map(String::from);

        let is_flake = node
            .get("original")
            .and_then(|o| o.get("type"))
            .and_then(|t| t.as_str())
            .map(|t| t == "github" || t == "git" || t == "path")
            .unwrap_or(true);

        result.push(FlakeInput {
            name: name.clone(),
            url,
            rev,
            is_flake,
        });
    }

    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

pub fn discover_external_modules(
    process: &NixProcess,
    repo_path: &Path,
) -> NixResult<Vec<ExternalModule>> {
    let inputs = discover_inputs(process, repo_path)?;

    let external_modules: Vec<ExternalModule> = inputs
        .into_iter()
        .filter(|input| {
            !input.name.starts_with("nixpkgs")
                && !input.name.starts_with("flake-utils")
        })
        .map(|input| ExternalModule {
            flake_name: input.name.clone(),
            url: input.url,
            rev: input.rev,
        })
        .collect();

    Ok(external_modules)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalModule {
    pub flake_name: String,
    pub url: String,
    pub rev: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_flake_metadata() {
        let metadata = serde_json::json!({
            "locks": {
                "root": "root",
                "nodes": {
                    "root": {
                        "inputs": {
                            "nixpkgs": "nixpkgs",
                            "nixvim": "nixvim",
                            "dms": "dms"
                        }
                    },
                    "nixpkgs": {
                        "locked": {"rev": "abc123"},
                        "original": {"type": "github", "url": "github:NixOS/nixpkgs/nixos-unstable"}
                    },
                    "nixvim": {
                        "locked": {"rev": "def456"},
                        "original": {"type": "github", "url": "github:nix-community/nixvim"}
                    },
                    "dms": {
                        "locked": {"rev": "ghi789"},
                        "original": {"type": "github", "url": "github:dankmaterialshell/dms"}
                    }
                }
            }
        });

        let inputs = parse_flake_inputs(&metadata).unwrap();
        assert_eq!(inputs.len(), 3);

        let dms = inputs.iter().find(|i| i.name == "dms").unwrap();
        assert_eq!(dms.url, "github:dankmaterialshell/dms");
        assert_eq!(dms.rev.as_deref(), Some("ghi789"));
    }

    #[test]
    fn parse_empty_metadata() {
        let metadata = serde_json::json!({});
        let inputs = parse_flake_inputs(&metadata).unwrap();
        assert!(inputs.is_empty());
    }

    #[test]
    fn external_modules_exclude_nixpkgs() {
        let metadata = serde_json::json!({
            "locks": {
                "root": "root",
                "nodes": {
                    "root": {
                        "inputs": {
                            "nixpkgs": "nixpkgs",
                            "flake-utils": "flake-utils",
                            "nixvim": "nixvim"
                        }
                    },
                    "nixpkgs": {
                        "locked": {"rev": "abc"},
                        "original": {"type": "github", "url": "github:NixOS/nixpkgs"}
                    },
                    "flake-utils": {
                        "locked": {"rev": "def"},
                        "original": {"type": "github", "url": "github:numtide/flake-utils"}
                    },
                    "nixvim": {
                        "locked": {"rev": "ghi"},
                        "original": {"type": "github", "url": "github:nix-community/nixvim"}
                    }
                }
            }
        });

        let inputs = parse_flake_inputs(&metadata).unwrap();
        let external: Vec<_> = inputs
            .into_iter()
            .filter(|i| !i.name.starts_with("nixpkgs") && !i.name.starts_with("flake-utils"))
            .collect();
        assert_eq!(external.len(), 1);
        assert_eq!(external[0].name, "nixvim");
    }
}
