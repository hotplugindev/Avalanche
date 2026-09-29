use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipStatus {
    Owned,
    Unowned,
    WrongOwner,
    Ambiguous,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnershipRole {
    Owner,
    Requester,
    Implementer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ownership {
    pub option_path: String,
    pub owner: String,
    pub status: OwnershipStatus,
}

impl Ownership {
    pub fn new(option_path: impl Into<String>, owner: impl Into<String>) -> Self {
        Self {
            option_path: option_path.into(),
            owner: owner.into(),
            status: OwnershipStatus::Owned,
        }
    }

    pub fn can_auto_edit(&self) -> bool {
        self.status == OwnershipStatus::Owned
    }
}
