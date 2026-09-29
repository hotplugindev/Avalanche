use avalanche_model::source::SourceLocation;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentKind {
    DirectLiteral,
    DirectExpression,
    GeneratedPattern,
    Computed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub path: String,
    pub kind: AssignmentKind,
    pub location: SourceLocation,
}

impl Assignment {
    pub fn is_auto_editable(&self) -> bool {
        matches!(
            self.kind,
            AssignmentKind::DirectLiteral | AssignmentKind::GeneratedPattern
        )
    }
}

pub struct SourceService;

impl Default for SourceService {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceService {
    pub fn new() -> Self {
        Self
    }
}
