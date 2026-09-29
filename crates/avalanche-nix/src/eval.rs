use std::path::Path;

use crate::error::{NixError, NixResult};
use crate::process::NixProcess;
use crate::value::decode_json;

pub fn eval_expr(process: &NixProcess, repo_path: &Path, expr: &str) -> NixResult<String> {
    let args = ["eval", "--json", "--expr", expr];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::EvalFailed {
            expression: expr.to_string(),
            stderr: output.stderr,
        });
    }

    Ok(output.stdout.trim().to_string())
}

pub fn eval_json(
    process: &NixProcess,
    repo_path: &Path,
    expr: &str,
) -> NixResult<serde_json::Value> {
    let raw = eval_expr(process, repo_path, expr)?;
    decode_json(&raw)
}

pub fn eval_attr(
    process: &NixProcess,
    repo_path: &Path,
    attr_path: &str,
) -> NixResult<serde_json::Value> {
    let args = ["eval", "--json", attr_path];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::EvalFailed {
            expression: attr_path.to_string(),
            stderr: output.stderr,
        });
    }

    decode_json(&output.stdout)
}

pub fn eval_attr_with_apply(
    process: &NixProcess,
    repo_path: &Path,
    attr_path: &str,
    apply_expr: &str,
) -> NixResult<serde_json::Value> {
    let args = ["eval", "--json", attr_path, "--apply", apply_expr];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::EvalFailed {
            expression: format!("{attr_path} --apply ..."),
            stderr: output.stderr,
        });
    }

    decode_json(&output.stdout)
}
