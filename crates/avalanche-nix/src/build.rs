use std::path::Path;

use crate::error::{NixError, NixResult};
use crate::process::NixProcess;

#[derive(Debug, Clone)]
pub struct BuildResult {
    pub success: bool,
    pub attribute: String,
    pub outputs: Vec<String>,
    pub stderr: String,
}

pub fn build(
    process: &NixProcess,
    repo_path: &Path,
    attribute: &str,
) -> NixResult<BuildResult> {
    let args = ["build", "--no-link", "--print-out-paths", attribute];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::BuildFailed {
            attribute: attribute.to_string(),
            stderr: output.stderr.clone(),
        });
    }

    let outputs: Vec<String> = output
        .stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    Ok(BuildResult {
        success: true,
        attribute: attribute.to_string(),
        outputs,
        stderr: output.stderr,
    })
}

pub fn build_dry_run(
    process: &NixProcess,
    repo_path: &Path,
    attribute: &str,
) -> NixResult<BuildResult> {
    let args = ["build", "--dry-run", attribute];
    let output = process.run(&args, repo_path)?;

    Ok(BuildResult {
        success: output.success,
        attribute: attribute.to_string(),
        outputs: Vec::new(),
        stderr: output.stderr,
    })
}
