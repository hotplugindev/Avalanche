use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NixError {
    #[error("nix command failed: {0}")]
    CommandFailed(String),
    #[error("nix evaluation failed: {0}")]
    EvalFailed(String),
    #[error("nix not found")]
    NixNotFound,
}

pub struct NixService {
    repo_path: String,
}

impl NixService {
    pub fn new(repo_path: impl Into<String>) -> Self {
        Self {
            repo_path: repo_path.into(),
        }
    }

    pub fn eval(&self, expr: &str) -> Result<String, NixError> {
        let output = Command::new("nix")
            .args(["eval", "--json", expr])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| NixError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(NixError::EvalFailed(stderr));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn flake_check(&self) -> Result<(), NixError> {
        let output = Command::new("nix")
            .args(["flake", "check", "--show-trace"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| NixError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(NixError::CommandFailed(stderr));
        }

        Ok(())
    }
}
