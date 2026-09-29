use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationTier {
    Structural,
    NixEval,
    FlakeCheck,
    Build,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationFinding {
    pub severity: ValidationSeverity,
    pub message: String,
    pub path: Option<String>,
    pub file: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationReport {
    pub tier: ValidationTier,
    pub findings: Vec<ValidationFinding>,
    pub passed: bool,
}

impl ValidationReport {
    pub fn new(tier: ValidationTier) -> Self {
        Self {
            tier,
            findings: Vec::new(),
            passed: true,
        }
    }

    pub fn add(&mut self, finding: ValidationFinding) {
        if finding.severity == ValidationSeverity::Error {
            self.passed = false;
        }
        self.findings.push(finding);
    }

    pub fn errors(&self) -> impl Iterator<Item = &ValidationFinding> {
        self.findings
            .iter()
            .filter(|f| f.severity == ValidationSeverity::Error)
    }
}
