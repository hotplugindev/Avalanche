use std::path::Path;

use avalanche_model::{NixType, OptionSchema, Scope};
use serde_json::Value;

use crate::error::{NixError, NixResult};
use crate::process::NixProcess;

pub fn extract_option(
    process: &NixProcess,
    repo_path: &Path,
    option_path: &str,
    scope: Scope,
    host: &str,
) -> NixResult<OptionSchema> {
    let apply_expr = r#"opt: {
  type = if opt ? type then builtins.toString (opt.type.description or opt.type.name or "unknown") else "unknown";
  default = if opt ? default then (let r = builtins.tryEval opt.default; in if r.success then r.value else null) else null;
  description = if opt ? description then opt.description else null;
  example = if opt ? example then (let r = builtins.tryEval opt.example; in if r.success then r.value else null) else null;
  internal = if opt ? internal then opt.internal else false;
  readOnly = if opt ? readOnly then opt.readOnly else false;
}"#;

    let attr = match scope {
        Scope::System => format!(".#nixosConfigurations.{host}.options.{option_path}"),
        Scope::Home => format!(".#homeConfigurations.{host}.options.{option_path}"),
    };

    let args = ["eval", "--json", &attr, "--apply", apply_expr];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::OptionExtractionFailed {
            path: option_path.to_string(),
            reason: output.stderr,
        });
    }

    let json: Value = serde_json::from_str(&output.stdout).map_err(|e| {
        NixError::OptionExtractionFailed {
            path: option_path.to_string(),
            reason: e.to_string(),
        }
    })?;

    parse_option_schema(&json, option_path, scope)
}

pub fn parse_option_schema(json: &Value, path: &str, scope: Scope) -> NixResult<OptionSchema> {
    let type_str = json
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let nix_type = parse_nix_type_string(type_str);

    let default = json
        .get("default")
        .filter(|v| !v.is_null())
        .map(|v| avalanche_model::JsonValue(v.clone()));

    let description = json
        .get("description")
        .and_then(|v| v.as_str())
        .map(String::from);

    let example = json
        .get("example")
        .filter(|v| !v.is_null())
        .map(|v| avalanche_model::JsonValue(v.clone()));

    let internal = json
        .get("internal")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let read_only = json
        .get("readOnly")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    Ok(OptionSchema {
        path: path.to_string(),
        scope,
        nix_type,
        default,
        description,
        example,
        internal,
        read_only,
    })
}

pub fn parse_nix_type_string(type_str: &str) -> NixType {
    let lower = type_str.to_lowercase();
    if lower.contains("boolean") || lower == "bool" {
        NixType::Bool
    } else if lower.contains("signed integer") || lower.contains("integer") || lower == "int" {
        NixType::Int
    } else if lower.contains("float") {
        NixType::Float
    } else if lower.contains("string") || lower == "str" {
        NixType::Str
    } else if lower.contains("path") && !lower.contains("package") {
        NixType::Path
    } else if lower.contains("list of") || lower.starts_with("list") {
        NixType::List
    } else if lower.starts_with("attribute set of") || lower.contains("attrs of") {
        NixType::AttrsOf
    } else if lower.contains("attribute set") || lower == "attrs" || lower == "attrset" {
        NixType::Attrs
    } else if lower.contains("submodule") {
        NixType::Submodule
    } else if lower.contains("package") {
        NixType::Package
    } else if lower.contains("enum") || lower.contains("one of") {
        NixType::Enum
    } else if lower.contains("function") || lower.contains("lambda") || lower == "raw" {
        NixType::Function
    } else {
        NixType::Unknown
    }
}

pub fn extract_option_value(
    process: &NixProcess,
    repo_path: &Path,
    option_path: &str,
    scope: Scope,
    host: &str,
) -> NixResult<Value> {
    let attr = match scope {
        Scope::System => format!(".#nixosConfigurations.{host}.config.{option_path}"),
        Scope::Home => format!(".#homeConfigurations.{host}.config.{option_path}"),
    };

    let args = ["eval", "--json", &attr];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::OptionExtractionFailed {
            path: option_path.to_string(),
            reason: output.stderr,
        });
    }

    serde_json::from_str(&output.stdout).map_err(|e| NixError::OptionExtractionFailed {
        path: option_path.to_string(),
        reason: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_type_strings() {
        assert_eq!(parse_nix_type_string("boolean"), NixType::Bool);
        assert_eq!(parse_nix_type_string("signed integer"), NixType::Int);
        assert_eq!(parse_nix_type_string("float"), NixType::Float);
        assert_eq!(parse_nix_type_string("string"), NixType::Str);
        assert_eq!(parse_nix_type_string("path"), NixType::Path);
        assert_eq!(parse_nix_type_string("list of ..."), NixType::List);
        assert_eq!(
            parse_nix_type_string("attribute set of ..."),
            NixType::AttrsOf
        );
        assert_eq!(parse_nix_type_string("attribute set"), NixType::Attrs);
        assert_eq!(parse_nix_type_string("submodule"), NixType::Submodule);
        assert_eq!(parse_nix_type_string("package"), NixType::Package);
        assert_eq!(parse_nix_type_string("one of ..."), NixType::Enum);
        assert_eq!(parse_nix_type_string("function"), NixType::Function);
        assert_eq!(parse_nix_type_string("something weird"), NixType::Unknown);
    }

    #[test]
    fn parse_schema_json() {
        let json = serde_json::json!({
            "type": "boolean",
            "default": false,
            "description": "Whether to enable the service",
            "example": null,
            "internal": false,
            "readOnly": false
        });
        let schema = parse_option_schema(&json, "services.foo.enable", Scope::System).unwrap();
        assert_eq!(schema.nix_type, NixType::Bool);
        assert_eq!(schema.path, "services.foo.enable");
        assert_eq!(
            schema.description.as_deref(),
            Some("Whether to enable the service")
        );
        assert!(!schema.internal);
        assert!(!schema.read_only);
    }

    #[test]
    fn parse_schema_missing_fields() {
        let json = serde_json::json!({"type": "unknown"});
        let schema = parse_option_schema(&json, "foo.bar", Scope::Home).unwrap();
        assert_eq!(schema.nix_type, NixType::Unknown);
        assert!(schema.default.is_none());
        assert!(schema.description.is_none());
    }

    #[test]
    fn parse_schema_null_default() {
        let json = serde_json::json!({"type": "boolean", "default": null});
        let schema = parse_option_schema(&json, "foo.bar", Scope::System).unwrap();
        assert!(schema.default.is_none());
    }

    #[test]
    fn parse_schema_enum_type() {
        let json = serde_json::json!({
            "type": "one of \"amd\", \"intel\", \"nvidia\"",
            "default": "amd",
            "description": "GPU vendor"
        });
        let schema = parse_option_schema(&json, "gb.host.hardware.gpu", Scope::System).unwrap();
        assert_eq!(schema.nix_type, NixType::Enum);
    }
}
