use std::path::Path;

use crate::error::{NixError, NixResult};
use crate::process::NixProcess;

#[derive(Debug, Clone)]
pub struct FlakeCheckResult {
    pub passed: bool,
    pub stdout: String,
    pub stderr: String,
}

pub fn flake_check(process: &NixProcess, repo_path: &Path) -> NixResult<FlakeCheckResult> {
    let args = ["flake", "check", "--no-build"];
    let output = process.run(&args, repo_path)?;

    Ok(FlakeCheckResult {
        passed: output.success,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

pub fn flake_check_strict(process: &NixProcess, repo_path: &Path) -> NixResult<()> {
    let args = ["flake", "check"];
    let output = process.run(&args, repo_path)?;

    if !output.success {
        return Err(NixError::FlakeCheckFailed {
            stderr: output.stderr,
        });
    }
    Ok(())
}
