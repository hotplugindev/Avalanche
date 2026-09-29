use serde::{Deserialize, Serialize};

use crate::source::SourceLocation;
use crate::validation::ValidationReport;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationIntent {
    SetOption {
        path: String,
        value: serde_json::Value,
        scope: String,
    },
    SetProfileDefault {
        path: String,
        value: serde_json::Value,
        profile: String,
    },
    SetHostOverride {
        path: String,
        value: serde_json::Value,
        host: String,
    },
    ResetHostOverride {
        path: String,
        host: String,
    },
    EnableCapability {
        capability: String,
        requester: String,
    },
    DisableCapability {
        capability: String,
        requester: String,
    },
    AddRequest {
        capability: String,
        requester: String,
    },
    RemoveRequest {
        capability: String,
        requester: String,
    },
    CreateModule {
        name: String,
        kind: String,
    },
    CreateHost {
        name: String,
    },
    CreateUser {
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionStatus {
    Planned,
    Validating,
    Ready,
    Applied,
    Rejected,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub intents: Vec<MutationIntent>,
    pub file_changes: Vec<FileChange>,
    pub validation: Option<ValidationReport>,
    pub diff: Option<String>,
    pub status: TransactionStatus,
}

impl Transaction {
    pub fn new(id: impl Into<String>, intents: Vec<MutationIntent>) -> Self {
        Self {
            id: id.into(),
            intents,
            file_changes: Vec::new(),
            validation: None,
            diff: None,
            status: TransactionStatus::Planned,
        }
    }

    pub fn touches(&self, location: &SourceLocation) -> bool {
        self.file_changes.iter().any(|c| c.path == location.file)
    }
}
