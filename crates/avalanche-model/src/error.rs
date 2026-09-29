use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("unknown scope: {0}")]
    UnknownScope(String),
    #[error("unknown host: {0}")]
    UnknownHost(String),
    #[error("unknown user: {0}")]
    UnknownUser(String),
    #[error("unknown capability: {0}")]
    UnknownCapability(String),
    #[error("unknown option: {0}")]
    UnknownOption(String),
    #[error("{0}")]
    Invalid(String),
}
