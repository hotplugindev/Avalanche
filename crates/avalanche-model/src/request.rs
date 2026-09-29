use serde::{Deserialize, Serialize};

use crate::capability::CapabilityId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub capability: CapabilityId,
    pub requester: String,
    pub source_file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedRequest {
    pub capability: CapabilityId,
    pub requesters: Vec<String>,
    pub active: bool,
}

impl ResolvedRequest {
    pub fn is_active(&self) -> bool {
        self.active
    }
}
