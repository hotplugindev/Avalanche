use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{NixError, NixResult};
use crate::process::NixProcess;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assertion {
    pub message: String,
    pub passed: bool,
}

const ASSERTIONS_APPLY: &str = r#"assertions: builtins.map (a: {
  message = a.message or "unknown assertion";
  passed = a.assertion;
}) assertions"#;

pub fn get_assertions(
    process: &NixProcess,
    repo_path: &Path,
    host: &str,
) -> NixResult<Vec<Assertion>> {
    let attr = format!(".#nixosConfigurations.{host}.config.assertions");
    let args = ["eval", "--json", &attr, "--apply", ASSERTIONS_APPLY];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::EvalFailed {
            expression: format!("{host} assertions"),
            stderr: output.stderr,
        });
    }

    let json: Value = serde_json::from_str(&output.stdout).map_err(|e| {
        NixError::JsonDecodeFailed {
            reason: e.to_string(),
        }
    })?;

    parse_assertions(&json)
}

pub fn parse_assertions(json: &Value) -> NixResult<Vec<Assertion>> {
    let arr = match json.as_array() {
        Some(a) => a,
        None => return Ok(Vec::new()),
    };

    Ok(arr
        .iter()
        .filter_map(|item| {
            let message = item.get("message")?.as_str()?.to_string();
            let passed = item.get("passed").and_then(|v| v.as_bool()).unwrap_or(false);
            Some(Assertion { message, passed })
        })
        .collect())
}

pub fn failed_assertions(assertions: &[Assertion]) -> Vec<&Assertion> {
    assertions.iter().filter(|a| !a.passed).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_assertions_list() {
        let json = serde_json::json!([
            {"message": "disk space sufficient", "passed": true},
            {"message": "kernel version valid", "passed": false}
        ]);
        let assertions = parse_assertions(&json).unwrap();
        assert_eq!(assertions.len(), 2);
        assert!(assertions[0].passed);
        assert!(!assertions[1].passed);
    }

    #[test]
    fn parse_empty_assertions() {
        let json = serde_json::json!([]);
        let assertions = parse_assertions(&json).unwrap();
        assert!(assertions.is_empty());
    }

    #[test]
    fn parse_null_assertions() {
        let json = serde_json::Value::Null;
        let assertions = parse_assertions(&json).unwrap();
        assert!(assertions.is_empty());
    }

    #[test]
    fn failed_assertions_filter() {
        let assertions = vec![
            Assertion {
                message: "ok".to_string(),
                passed: true,
            },
            Assertion {
                message: "bad".to_string(),
                passed: false,
            },
        ];
        let failed = failed_assertions(&assertions);
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].message, "bad");
    }
}
