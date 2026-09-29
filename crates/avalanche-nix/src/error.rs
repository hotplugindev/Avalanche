use thiserror::Error;

#[derive(Debug, Error)]
pub enum NixError {
    #[error("nix binary not found on PATH")]
    NixNotFound,
    #[error("failed to spawn nix process: {reason}")]
    SpawnFailed { reason: String },
    #[error("nix process timed out after {seconds}s")]
    Timeout { seconds: u64 },
    #[error("nix eval failed for expression '{expression}': {stderr}")]
    EvalFailed { expression: String, stderr: String },
    #[error("nix flake check failed: {stderr}")]
    FlakeCheckFailed { stderr: String },
    #[error("nix build failed for '{attribute}': {stderr}")]
    BuildFailed { attribute: String, stderr: String },
    #[error("failed to decode nix JSON output: {reason}")]
    JsonDecodeFailed { reason: String },
    #[error("option extraction failed for '{path}': {reason}")]
    OptionExtractionFailed { path: String, reason: String },
    #[error("host evaluation failed for '{host}': {reason}")]
    HostEvalFailed { host: String, reason: String },
    #[error("home-manager evaluation failed for host '{host}', user '{user}': {reason}")]
    HomeEvalFailed {
        host: String,
        user: String,
        reason: String,
    },
    #[error("flake input discovery failed: {reason}")]
    FlakeDiscoveryFailed { reason: String },
    #[error("repository path not found: {path}")]
    RepoNotFound { path: String },
}

pub type NixResult<T> = Result<T, NixError>;
