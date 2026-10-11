use serde::{Deserialize, Serialize};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("git command failed: {command}: {stderr}")]
    CommandFailed { command: String, stderr: String },
    #[error("not a git repository: {path}")]
    NotARepo { path: String },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub staged: Vec<String>,
    pub modified: Vec<String>,
    pub untracked: Vec<String>,
    pub deleted: Vec<String>,
    pub conflicted: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub is_current: bool,
    pub remote: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConflictInfo {
    pub path: String,
    pub base: Option<String>,
    pub ours: Option<String>,
    pub theirs: Option<String>,
}

pub struct GitService {
    repo_path: String,
}

impl GitService {
    pub fn init(path: &str) -> Result<(), GitError> {
        let out = Command::new("git")
            .args(["init", path])
            .output()
            .map_err(GitError::Io)?;
        if !out.status.success() {
            return Err(GitError::CommandFailed {
                command: format!("git init {path}"),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            });
        }

        let out = Command::new("git")
            .args(["-C", path, "add", "-A"])
            .output()
            .map_err(GitError::Io)?;
        if !out.status.success() {
            return Err(GitError::CommandFailed {
                command: format!("git -C {path} add -A"),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            });
        }

        let out = Command::new("git")
            .args([
                "-C",
                path,
                "-c",
                "user.name=Avalanche",
                "-c",
                "user.email=avalanche@localhost",
                "commit",
                "-m",
                "Initialize Avalanche repository",
                "--allow-empty",
            ])
            .output()
            .map_err(GitError::Io)?;
        if !out.status.success() {
            return Err(GitError::CommandFailed {
                command: format!("git -C {path} commit"),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            });
        }

        Ok(())
    }

    pub fn new(repo_path: impl Into<String>) -> Self {
        Self {
            repo_path: repo_path.into(),
        }
    }

    fn run(&self, args: &[&str]) -> Result<String, GitError> {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()?;
        if !output.status.success() {
            return Err(GitError::CommandFailed {
                command: format!("git {}", args.join(" ")),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub fn head(&self) -> Result<String, GitError> {
        self.run(&["rev-parse", "HEAD"])
            .map(|s| s.trim().to_string())
    }

    pub fn is_dirty(&self) -> Result<bool, GitError> {
        let out = self.run(&["status", "--porcelain"])?;
        Ok(!out.trim().is_empty())
    }

    pub fn diff(&self) -> Result<String, GitError> {
        self.run(&["diff"])
    }

    pub fn status(&self) -> Result<GitStatus, GitError> {
        let out = self.run(&["status", "--porcelain=v2", "--branch"])?;
        parse_status(&out)
    }

    pub fn history(&self, max_count: usize) -> Result<Vec<GitCommit>, GitError> {
        let fmt = "%H%n%h%n%s%n%an%n%at";
        let out = self.run(&[
            "log",
            &format!("--format={fmt}"),
            "-n",
            &max_count.to_string(),
        ])?;
        let lines: Vec<&str> = out.lines().collect();
        let mut commits = Vec::new();
        for chunk in lines.chunks(5) {
            if chunk.len() < 5 {
                break;
            }
            commits.push(GitCommit {
                hash: chunk[0].to_string(),
                short_hash: chunk[1].to_string(),
                message: chunk[2].to_string(),
                author: chunk[3].to_string(),
                timestamp: chunk[4].parse().unwrap_or(0),
            });
        }
        Ok(commits)
    }

    pub fn branches(&self) -> Result<Vec<GitBranch>, GitError> {
        let out = self.run(&[
            "for-each-ref",
            "--format=%(refname)%09%(HEAD)%09%(upstream:short)",
            "refs/heads/",
            "refs/remotes/",
        ])?;
        let mut result = Vec::new();
        for line in out.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 3 {
                continue;
            }
            let refname = parts[0];
            let is_current = parts[1] == "*";
            let upstream = parts[2];

            if let Some(name) = refname.strip_prefix("refs/heads/") {
                result.push(GitBranch {
                    name: name.to_string(),
                    is_current,
                    remote: if upstream.is_empty() {
                        None
                    } else {
                        Some(upstream.to_string())
                    },
                });
            } else if let Some(name) = refname.strip_prefix("refs/remotes/") {
                let remote_name = name.split('/').next().unwrap_or(name);
                result.push(GitBranch {
                    name: name.to_string(),
                    is_current: false,
                    remote: Some(remote_name.to_string()),
                });
            }
        }
        Ok(result)
    }

    pub fn current_branch(&self) -> Result<Option<String>, GitError> {
        let out = self.run(&["rev-parse", "--abbrev-ref", "HEAD"])?;
        let name = out.trim().to_string();
        if name == "HEAD" {
            Ok(None)
        } else {
            Ok(Some(name))
        }
    }

    pub fn create_branch(&self, name: &str) -> Result<(), GitError> {
        self.run(&["checkout", "-b", name])?;
        Ok(())
    }

    pub fn checkout_branch(&self, name: &str) -> Result<(), GitError> {
        self.run(&["checkout", name])?;
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<String, GitError> {
        self.run(&["add", "-A"])?;
        self.run(&["commit", "-m", message])?;
        self.head()
    }

    pub fn pull(&self) -> Result<String, GitError> {
        let output = Command::new("git")
            .args(["pull"])
            .current_dir(&self.repo_path)
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            return Err(GitError::CommandFailed {
                command: "git pull".to_string(),
                stderr: format!("{stdout}{stderr}"),
            });
        }
        Ok(format!("{stdout}{stderr}"))
    }

    pub fn push(&self) -> Result<String, GitError> {
        let output = Command::new("git")
            .args(["push"])
            .current_dir(&self.repo_path)
            .output()?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            return Err(GitError::CommandFailed {
                command: "git push".to_string(),
                stderr: format!("{stdout}{stderr}"),
            });
        }
        Ok(format!("{stdout}{stderr}"))
    }

    pub fn rollback(&self, paths: &[String]) -> Result<(), GitError> {
        for path in paths {
            self.run(&["checkout", "HEAD", "--", path])?;
        }
        Ok(())
    }

    pub fn conflicts(&self) -> Result<Vec<ConflictInfo>, GitError> {
        let out = self.run(&["diff", "--name-only", "--diff-filter=U"])?;
        let mut result = Vec::new();
        for path in out.lines() {
            if path.is_empty() {
                continue;
            }
            let base = self.show_stage(1, path).ok();
            let ours = self.show_stage(2, path).ok();
            let theirs = self.show_stage(3, path).ok();
            result.push(ConflictInfo {
                path: path.to_string(),
                base,
                ours,
                theirs,
            });
        }
        Ok(result)
    }

    fn show_stage(&self, stage: u8, path: &str) -> Result<String, GitError> {
        let ref_spec = format!(":{stage}:{path}");
        self.run(&["show", &ref_spec])
    }
}

fn parse_status(out: &str) -> Result<GitStatus, GitError> {
    let mut branch = None;
    let mut ahead = 0u32;
    let mut behind = 0u32;
    let mut staged = Vec::new();
    let mut modified = Vec::new();
    let mut untracked = Vec::new();
    let mut deleted = Vec::new();
    let mut conflicted = Vec::new();

    for line in out.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            if rest != "(detached)" {
                branch = Some(rest.to_string());
            }
        } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
            for part in rest.split_whitespace() {
                if let Some(a) = part.strip_prefix('+') {
                    ahead = a.parse().unwrap_or(0);
                } else if let Some(b) = part.strip_prefix('-') {
                    behind = b.parse().unwrap_or(0);
                }
            }
        } else if line.starts_with("1 ") {
            let parts: Vec<&str> = line.splitn(9, ' ').collect();
            if parts.len() < 9 {
                continue;
            }
            let xy = parts[1];
            let path = parts[8];
            classify_xy(xy, path, &mut staged, &mut modified, &mut deleted);
        } else if line.starts_with("2 ") {
            let parts: Vec<&str> = line.splitn(10, ' ').collect();
            if parts.len() < 10 {
                continue;
            }
            let xy = parts[1];
            let path = parts[9].split('\0').next().unwrap_or(parts[9]);
            classify_xy(xy, path, &mut staged, &mut modified, &mut deleted);
        } else if line.starts_with("u ") {
            let parts: Vec<&str> = line.splitn(10, ' ').collect();
            if parts.len() >= 10 {
                conflicted.push(parts[9].to_string());
            }
        } else if let Some(path) = line.strip_prefix("? ") {
            untracked.push(path.to_string());
        }
    }

    Ok(GitStatus {
        branch,
        ahead,
        behind,
        staged,
        modified,
        untracked,
        deleted,
        conflicted,
    })
}

fn classify_xy(
    xy: &str,
    path: &str,
    staged: &mut Vec<String>,
    modified: &mut Vec<String>,
    deleted: &mut Vec<String>,
) {
    let mut chars = xy.chars();
    let x = chars.next().unwrap_or('.');
    let y = chars.next().unwrap_or('.');

    if x != '.' && x != '?' {
        staged.push(path.to_string());
    }
    if y == 'M' {
        modified.push(path.to_string());
    }
    if x == 'D' || y == 'D' {
        deleted.push(path.to_string());
    }
}
