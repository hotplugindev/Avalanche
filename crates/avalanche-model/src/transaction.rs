use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::repository::Fingerprint;
use crate::scope::Scope;
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

    pub fn from_plan(
        id: impl Into<String>,
        intents: Vec<MutationIntent>,
        plan: &MutationPlan,
    ) -> Self {
        Self {
            id: id.into(),
            intents,
            file_changes: plan.file_changes.clone(),
            validation: None,
            diff: Some(plan.diff.clone()),
            status: plan.status.clone(),
        }
    }

    pub fn touches(&self, location: &SourceLocation) -> bool {
        self.file_changes.iter().any(|c| c.path == location.file)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditKind {
    ReplaceValue,
    RemoveBinding,
    InsertBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderHint {
    Literal,
    MkDefault,
    MkForce,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedEdit {
    pub file: String,
    pub option_path: String,
    pub kind: EditKind,
    pub value: Option<serde_json::Value>,
    pub render: RenderHint,
    pub layer: Layer,
    pub scope: Scope,
    pub host: Option<String>,
    pub user: Option<String>,
    pub profile: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MutationPlan {
    pub edits: Vec<ResolvedEdit>,
    pub file_changes: Vec<FileChange>,
    pub diff: String,
    pub fingerprint: Fingerprint,
    pub status: TransactionStatus,
}
