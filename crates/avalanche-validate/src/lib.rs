use avalanche_model::validation::{ValidationReport, ValidationTier};

pub struct ValidationService;

impl ValidationService {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_structural(&self) -> ValidationReport {
        ValidationReport::new(ValidationTier::Structural)
    }
}
