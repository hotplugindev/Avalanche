use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use crate::error::{NixError, NixResult};

#[derive(Debug, Clone)]
pub struct NixProcessConfig {
    pub nix_binary: String,
    pub timeout: Duration,
    pub extra_args: Vec<String>,
}

impl Default for NixProcessConfig {
    fn default() -> Self {
        Self {
            nix_binary: "nix".to_string(),
            timeout: Duration::from_secs(300),
            extra_args: Vec::new(),
        }
    }
}

impl NixProcessConfig {
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_extra_args(mut self, args: Vec<String>) -> Self {
        self.extra_args = args;
        self
    }
}

pub struct NixProcess {
    config: NixProcessConfig,
}

impl Default for NixProcess {
    fn default() -> Self {
        Self::new(NixProcessConfig::default())
    }
}

impl NixProcess {
    pub fn new(config: NixProcessConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &NixProcessConfig {
        &self.config
    }

    pub fn run(&self, args: &[&str], workdir: &Path) -> NixResult<ProcessOutput> {
        let mut cmd = Command::new(&self.config.nix_binary);
        cmd.args(&self.config.extra_args);
        cmd.args(args);
        cmd.current_dir(workdir);
        cmd.env("NIX_CONFIG", "experimental-features = nix-command flakes");

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                NixError::NixNotFound
            } else {
                NixError::SpawnFailed {
                    reason: e.to_string(),
                }
            }
        })?;

        Ok(ProcessOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            code: output.status.code(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ProcessOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub code: Option<i32>,
}

impl ProcessOutput {
    pub fn ensure_success(self, context: NixError) -> NixResult<String> {
        if self.success {
            Ok(self.stdout)
        } else {
            Err(context)
        }
    }
}

pub fn resolve_repo_path(path: &str) -> NixResult<PathBuf> {
    let p = PathBuf::from(path);
    if !p.exists() {
        return Err(NixError::RepoNotFound {
            path: path.to_string(),
        });
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = NixProcessConfig::default();
        assert_eq!(config.nix_binary, "nix");
        assert_eq!(config.timeout, Duration::from_secs(300));
        assert!(config.extra_args.is_empty());
    }

    #[test]
    fn config_builders() {
        let config = NixProcessConfig::default()
            .with_timeout(Duration::from_secs(60))
            .with_extra_args(vec!["--offline".to_string()]);
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.extra_args, vec!["--offline"]);
    }

    #[test]
    fn resolve_existing_path() {
        let result = resolve_repo_path("/tmp");
        assert!(result.is_ok());
    }

    #[test]
    fn resolve_nonexistent_path() {
        let result = resolve_repo_path("/nonexistent/path/xyz");
        assert!(result.is_err());
    }
}
