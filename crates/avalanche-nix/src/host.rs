use std::path::Path;

use serde_json::Value;

use crate::error::{NixError, NixResult};
use crate::process::NixProcess;

#[derive(Debug, Clone)]
pub struct HostEvalResult {
    pub host: String,
    pub system: String,
    pub config: Value,
}

pub fn list_hosts(process: &NixProcess, repo_path: &Path) -> NixResult<Vec<String>> {
    let apply = "x: builtins.attrNames x";
    let args = ["eval", "--json", ".#nixosConfigurations", "--apply", apply];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::HostEvalFailed {
            host: "*".to_string(),
            reason: output.stderr,
        });
    }

    let json: Value = serde_json::from_str(&output.stdout).map_err(|e| NixError::JsonDecodeFailed {
        reason: e.to_string(),
    })?;

    Ok(json
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default())
}

pub fn eval_host(
    process: &NixProcess,
    repo_path: &Path,
    host: &str,
) -> NixResult<HostEvalResult> {
    let attr = format!(".#nixosConfigurations.{host}");
    let apply = r#"cfg: {
  system = (if cfg ? pkgs.stdenv.hostPlatform.system then cfg.pkgs.stdenv.hostPlatform.system else "unknown");
}"#;
    let args = ["eval", "--json", &attr, "--apply", apply];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::HostEvalFailed {
            host: host.to_string(),
            reason: output.stderr,
        });
    }

    let json: Value = serde_json::from_str(&output.stdout).map_err(|e| NixError::HostEvalFailed {
        host: host.to_string(),
        reason: e.to_string(),
    })?;

    let system = json
        .get("system")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(HostEvalResult {
        host: host.to_string(),
        system,
        config: Value::Null,
    })
}

pub fn eval_host_option(
    process: &NixProcess,
    repo_path: &Path,
    host: &str,
    option_path: &str,
) -> NixResult<Value> {
    let attr = format!(".#nixosConfigurations.{host}.config.{option_path}");
    let args = ["eval", "--json", &attr];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::OptionExtractionFailed {
            path: format!("{host}.{option_path}"),
            reason: output.stderr,
        });
    }

    serde_json::from_str(&output.stdout).map_err(|e| NixError::JsonDecodeFailed {
        reason: e.to_string(),
    })
}

pub fn list_home_configs(process: &NixProcess, repo_path: &Path) -> NixResult<Vec<String>> {
    let apply = "x: builtins.attrNames x";
    let args = ["eval", "--json", ".#homeConfigurations", "--apply", apply];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::HomeEvalFailed {
            host: "*".to_string(),
            user: "*".to_string(),
            reason: output.stderr,
        });
    }

    let json: Value = serde_json::from_str(&output.stdout).map_err(|e| NixError::JsonDecodeFailed {
        reason: e.to_string(),
    })?;

    Ok(json
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default())
}

pub fn eval_home_manager(
    process: &NixProcess,
    repo_path: &Path,
    host: &str,
    user: &str,
) -> NixResult<Value> {
    let key = format!("{user}@{host}");
    let attr = format!(r#".#homeConfigurations."{key}""#);
    let apply = "cfg: builtins.tryEval cfg";
    let args = ["eval", "--json", &attr, "--apply", apply];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::HomeEvalFailed {
            host: host.to_string(),
            user: user.to_string(),
            reason: output.stderr,
        });
    }

    serde_json::from_str(&output.stdout).map_err(|e| NixError::HomeEvalFailed {
        host: host.to_string(),
        user: user.to_string(),
        reason: e.to_string(),
    })
}

pub fn eval_home_manager_option(
    process: &NixProcess,
    repo_path: &Path,
    host: &str,
    user: &str,
    option_path: &str,
) -> NixResult<Value> {
    let key = format!("{user}@{host}");
    let attr = format!(r#".#homeConfigurations."{key}".config.{option_path}"#);
    let args = ["eval", "--json", &attr];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::OptionExtractionFailed {
            path: format!("{user}@{host}.{option_path}"),
            reason: output.stderr,
        });
    }

    serde_json::from_str(&output.stdout).map_err(|e| NixError::JsonDecodeFailed {
        reason: e.to_string(),
    })
}
