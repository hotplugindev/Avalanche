use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("filesystem walk failed at '{path}': {reason}")]
    WalkFailed { path: String, reason: String },
    #[error("parse failed for '{file}': {reason}")]
    ParseFailed { file: String, reason: String },
    #[error("nix evaluation failed during semantic index: {reason}")]
    EvalFailed { reason: String },
    #[error("fingerprint computation failed: {reason}")]
    FingerprintFailed { reason: String },
    #[error("repository root not found: {path}")]
    RepoNotFound { path: String },
    #[error("{0}")]
    Other(String),
}

pub type IndexResult<T> = Result<T, IndexError>;
