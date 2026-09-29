use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("git command failed: {0}")]
    CommandFailed(String),
    #[error("not a git repository")]
    NotARepository,
}

pub struct GitService {
    repo_path: String,
}

impl GitService {
    pub fn new(repo_path: impl Into<String>) -> Self {
        Self {
            repo_path: repo_path.into(),
        }
    }

    pub fn head(&self) -> Result<String, GitError> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(GitError::NotARepository);
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub fn is_dirty(&self) -> Result<bool, GitError> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(GitError::NotARepository);
        }

        Ok(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
    }

    pub fn diff(&self) -> Result<String, GitError> {
        let output = Command::new("git")
            .args(["diff"])
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GitError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(GitError::NotARepository);
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
