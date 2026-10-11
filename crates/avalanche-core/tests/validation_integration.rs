use std::path::{Path, PathBuf};

use avalanche_core::ConfigurationEngine;
use avalanche_model::validation::ValidationTier;
use avalanche_model::{MutationIntent, Repository, TransactionStatus};

fn write_file(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn fixture(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("avalanche-valid-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    write_file(
        &dir,
        "modules/capabilities/system/audio/pipewire.nix",
        "{\n  services.pipewire.enable = true;\n}\n",
    );
    write_file(
        &dir,
        "hosts/pc/default.nix",
        "{\n  networking.hostName = \"pc-host\";\n}\n",
    );
    dir
}

fn repo(root: &Path) -> Repository {
    Repository::new(root.display().to_string())
}

#[test]
fn applied_transaction_carries_explicit_structural_validation_state() {
    let dir = fixture("state");
    let r = repo(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&r).unwrap();

    let txn = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::SetOption {
                path: "services.pipewire.enable".to_string(),
                value: serde_json::json!(false),
                scope: "system".to_string(),
            }],
        )
        .unwrap();

    assert_eq!(txn.status, TransactionStatus::Applied);
    let report = txn
        .validation
        .expect("every applied transaction must carry a validation state");
    assert_eq!(report.tier, ValidationTier::Structural);
    assert!(report.passed);
    assert!(report.findings.is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn validate_transaction_is_deterministic_when_no_host_files_touched() {
    let dir = fixture("deterministic");
    let r = repo(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&r).unwrap();

    let txn = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::SetOption {
                path: "services.pipewire.enable".to_string(),
                value: serde_json::json!(true),
                scope: "system".to_string(),
            }],
        )
        .unwrap();

    assert!(txn.file_changes.is_empty());

    let report = engine.validate_transaction(&txn);
    assert_eq!(report.tier, ValidationTier::NixEval);
    assert!(report.passed);
    assert!(report.findings.is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn execute_mutation_structural_only_skips_nix() {
    let dir = fixture("structural-only");
    let r = repo(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&r).unwrap();

    let txn = engine
        .execute_mutation_with_tier(
            &index,
            vec![MutationIntent::SetHostOverride {
                path: "networking.hostName".to_string(),
                value: serde_json::json!("renamed"),
                host: "pc".to_string(),
            }],
            ValidationTier::Structural,
        )
        .unwrap();

    let report = txn.validation.expect("structural tier attaches a report");
    assert_eq!(report.tier, ValidationTier::Structural);
    assert!(
        report.findings.is_empty(),
        "structural tier must not invoke nix: {:?}",
        report.findings
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn execute_mutation_nix_eval_tier_attaches_report() {
    let dir = fixture("nix-eval-tier");
    let r = repo(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&r).unwrap();

    let txn = engine
        .execute_mutation_with_tier(
            &index,
            vec![MutationIntent::SetHostOverride {
                path: "networking.hostName".to_string(),
                value: serde_json::json!("renamed"),
                host: "pc".to_string(),
            }],
            ValidationTier::NixEval,
        )
        .unwrap();

    assert_eq!(txn.status, TransactionStatus::Applied);
    let report = txn
        .validation
        .expect("nix eval tier must attach a validation report");
    assert_eq!(report.tier, ValidationTier::NixEval);

    let _ = std::fs::remove_dir_all(&dir);
}
