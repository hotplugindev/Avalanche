use avalanche_model::validation::{ValidationReport, ValidationTier};

pub struct ValidationService;

impl Default for ValidationService {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationService {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_structural(&self) -> ValidationReport {
        ValidationReport::new(ValidationTier::Structural)
    }
}
