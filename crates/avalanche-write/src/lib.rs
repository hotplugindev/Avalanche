use avalanche_model::{MutationIntent, Transaction, TransactionStatus};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("ownership check failed: {0}")]
    OwnershipDenied(String),
    #[error("stale index: repository changed after indexing")]
    StaleIndex,
    #[error("validation failed: {0}")]
    ValidationFailed(String),
    #[error("edit failed: {0}")]
    EditFailed(String),
}

pub struct WriteService;

impl Default for WriteService {
    fn default() -> Self {
        Self::new()
    }
}

impl WriteService {
    pub fn new() -> Self {
        Self
    }

    pub fn plan(&self, intents: Vec<MutationIntent>) -> Result<Transaction, WriteError> {
        let id = format!("txn-{}", uuid_v4());
        Ok(Transaction::new(id, intents))
    }

    pub fn apply(&mut self, transaction: &mut Transaction) -> Result<(), WriteError> {
        transaction.status = TransactionStatus::Applied;
        Ok(())
    }
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{:x}{:x}", secs, nanos)
}
