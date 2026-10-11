use avalanche_model::validation::{
    ValidationFinding, ValidationReport, ValidationSeverity, ValidationTier,
};
use avalanche_nix::NixService;

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

    pub fn validate_structural(&self, files: &[(&str, &str)]) -> ValidationReport {
        let mut report = ValidationReport::new(ValidationTier::Structural);
        for (path, content) in files {
            match avalanche_source::parse(content) {
                Ok(_) => {}
                Err(e) => {
                    report.add(ValidationFinding {
                        severity: ValidationSeverity::Error,
                        message: format!("parse error in {path}: {e}"),
                        path: Some((*path).to_string()),
                        file: Some((*path).to_string()),
                    });
                }
            }
        }
        report
    }

    pub fn validate_nix_eval(&self, nix: &NixService, hosts: &[String]) -> ValidationReport {
        let mut report = ValidationReport::new(ValidationTier::NixEval);
        for host in hosts {
            match nix.eval_host(host) {
                Ok(_) => {}
                Err(avalanche_nix::NixError::NixNotFound) => {
                    report.add(ValidationFinding {
                        severity: ValidationSeverity::Warning,
                        message: "nix binary not available; Tier 2 evaluation skipped".to_string(),
                        path: None,
                        file: None,
                    });
                    return report;
                }
                Err(e) => {
                    report.add(ValidationFinding {
                        severity: ValidationSeverity::Error,
                        message: format!("nix eval failed for host '{host}': {e}"),
                        path: None,
                        file: None,
                    });
                }
            }
        }
        report
    }

    pub fn validate_assertions(&self, nix: &NixService, host: &str) -> ValidationReport {
        let mut report = ValidationReport::new(ValidationTier::NixEval);
        match nix.get_assertions(host) {
            Ok(assertions) => {
                for a in &assertions {
                    if !a.passed {
                        report.add(ValidationFinding {
                            severity: ValidationSeverity::Error,
                            message: format!("assertion failed on {host}: {}", a.message),
                            path: None,
                            file: None,
                        });
                    }
                }
            }
            Err(avalanche_nix::NixError::NixNotFound) => {
                report.add(ValidationFinding {
                    severity: ValidationSeverity::Warning,
                    message: "nix binary not available; assertion check skipped".to_string(),
                    path: None,
                    file: None,
                });
            }
            Err(e) => {
                report.add(ValidationFinding {
                    severity: ValidationSeverity::Warning,
                    message: format!("could not evaluate assertions for {host}: {e}"),
                    path: None,
                    file: None,
                });
            }
        }
        report
    }

    pub fn validate_flake_check(&self, nix: &NixService) -> ValidationReport {
        let mut report = ValidationReport::new(ValidationTier::FlakeCheck);
        match nix.flake_check() {
            Ok(result) if !result.passed => {
                report.add(ValidationFinding {
                    severity: ValidationSeverity::Error,
                    message: format!("nix flake check failed: {}", result.stderr),
                    path: None,
                    file: None,
                });
            }
            Ok(_) => {}
            Err(avalanche_nix::NixError::NixNotFound) => {
                report.add(ValidationFinding {
                    severity: ValidationSeverity::Warning,
                    message: "nix binary not available; flake check skipped".to_string(),
                    path: None,
                    file: None,
                });
            }
            Err(e) => {
                report.add(ValidationFinding {
                    severity: ValidationSeverity::Error,
                    message: format!("nix flake check error: {e}"),
                    path: None,
                    file: None,
                });
            }
        }
        report
    }

    pub fn validate_build(&self, nix: &NixService, hosts: &[String]) -> ValidationReport {
        let mut report = ValidationReport::new(ValidationTier::Build);
        for host in hosts {
            let attr = format!(".#nixosConfigurations.{host}.config.system.build.toplevel");
            match nix.build_dry_run(&attr) {
                Ok(result) if !result.success => {
                    report.add(ValidationFinding {
                        severity: ValidationSeverity::Error,
                        message: format!(
                            "nix build dry-run failed for host '{host}': {}",
                            result.stderr
                        ),
                        path: None,
                        file: None,
                    });
                }
                Ok(_) => {}
                Err(avalanche_nix::NixError::NixNotFound) => {
                    report.add(ValidationFinding {
                        severity: ValidationSeverity::Warning,
                        message: "nix binary not available; Tier 4 build skipped".to_string(),
                        path: None,
                        file: None,
                    });
                    return report;
                }
                Err(e) => {
                    report.add(ValidationFinding {
                        severity: ValidationSeverity::Error,
                        message: format!("nix build dry-run error for host '{host}': {e}"),
                        path: None,
                        file: None,
                    });
                }
            }
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structural_valid_passes() {
        let svc = ValidationService::new();
        let files = vec![("test.nix", "{ enable = true; }")];
        let report = svc.validate_structural(&files);
        assert!(report.passed);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn structural_invalid_reports_error() {
        let svc = ValidationService::new();
        let files = vec![("bad.nix", "{ a = ;; }")];
        let report = svc.validate_structural(&files);
        assert!(!report.passed);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, ValidationSeverity::Error);
        assert!(report.findings[0].message.contains("bad.nix"));
    }

    #[test]
    fn structural_multiple_files_all_checked() {
        let svc = ValidationService::new();
        let files = vec![
            ("good.nix", "{ x = 1; }"),
            ("bad.nix", "{ y = ;; }"),
            ("also_good.nix", "{ z = true; }"),
        ];
        let report = svc.validate_structural(&files);
        assert!(!report.passed);
        assert_eq!(report.findings.len(), 1);
        assert!(report.findings[0]
            .file
            .as_deref()
            .unwrap()
            .contains("bad.nix"));
    }

    #[test]
    fn structural_empty_input_passes() {
        let svc = ValidationService::new();
        let report = svc.validate_structural(&[]);
        assert!(report.passed);
    }

    #[test]
    fn structural_multiline_valid() {
        let svc = ValidationService::new();
        let content = "{\n  services.pipewire.enable = true;\n  networking.hostName = \"pc\";\n}\n";
        let files = vec![("m.nix", content)];
        let report = svc.validate_structural(&files);
        assert!(report.passed);
    }

    #[test]
    fn tier_is_structural() {
        let svc = ValidationService::new();
        let report = svc.validate_structural(&[]);
        assert_eq!(report.tier, ValidationTier::Structural);
    }

    #[test]
    fn structural_preserves_file_in_finding() {
        let svc = ValidationService::new();
        let files = vec![("hosts/pc/default.nix", "{ a = ;; }")];
        let report = svc.validate_structural(&files);
        assert_eq!(
            report.findings[0].file.as_deref().unwrap(),
            "hosts/pc/default.nix"
        );
        assert_eq!(
            report.findings[0].path.as_deref().unwrap(),
            "hosts/pc/default.nix"
        );
    }

    fn missing_nix() -> NixService {
        let config = avalanche_nix::NixProcessConfig {
            nix_binary: "avalanche-definitely-not-a-nix-binary".to_string(),
            ..Default::default()
        };
        NixService::with_config("/tmp", config)
    }

    #[test]
    fn validate_build_returns_warning_when_nix_missing() {
        let svc = ValidationService::new();
        let report = svc.validate_build(&missing_nix(), &["pc".to_string()]);
        assert_eq!(report.tier, ValidationTier::Build);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, ValidationSeverity::Warning);
        assert!(report.passed);
    }

    #[test]
    fn validate_build_empty_hosts_passes() {
        let svc = ValidationService::new();
        let report = svc.validate_build(&missing_nix(), &[]);
        assert_eq!(report.tier, ValidationTier::Build);
        assert!(report.passed);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn validate_flake_check_returns_warning_when_nix_missing() {
        let svc = ValidationService::new();
        let report = svc.validate_flake_check(&missing_nix());
        assert_eq!(report.tier, ValidationTier::FlakeCheck);
        assert_eq!(report.findings[0].severity, ValidationSeverity::Warning);
        assert!(report.passed);
    }
}
