use avalanche_model::validation::{ValidationFinding, ValidationSeverity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diagnosis {
    pub findings: Vec<ValidationFinding>,
}

impl Default for Diagnosis {
    fn default() -> Self {
        Self::new()
    }
}

impl Diagnosis {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
        }
    }

    pub fn add(&mut self, finding: ValidationFinding) {
        self.findings.push(finding);
    }

    pub fn has_errors(&self) -> bool {
        self.findings
            .iter()
            .any(|f| f.severity == ValidationSeverity::Error)
    }
}

pub struct DoctorService;

impl Default for DoctorService {
    fn default() -> Self {
        Self::new()
    }
}

impl DoctorService {
    pub fn new() -> Self {
        Self
    }

    pub fn diagnose(&self) -> Diagnosis {
        Diagnosis::new()
    }
}
