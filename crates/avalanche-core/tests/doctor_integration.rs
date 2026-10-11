use std::path::{Path, PathBuf};

use avalanche_core::ConfigurationEngine;
use avalanche_doctor::RepairAction;
use avalanche_index::IndexService;
use avalanche_model::Repository;

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/provenance")
}

#[test]
fn doctor_detects_structural_problems_end_to_end() {
    let repo_path = fixture_path();
    if !repo_path.exists() {
        return;
    }
    let repository = Repository::new(repo_path.display().to_string());
    let index = IndexService::new().build_static_only(&repository).unwrap();
    if index.static_index.has_parse_errors() {
        return;
    }

    let engine = ConfigurationEngine::new(repo_path.display().to_string());
    let diag = engine.diagnose(&index);

    assert!(
        diag.has_errors(),
        "doctor must report errors for this fixture"
    );

    assert!(diag
        .findings
        .iter()
        .any(|f| f.message.contains("gb.schemaVersion")));

    let aggregate_errors = diag
        .findings
        .iter()
        .filter(|f| f.message.contains("not imported by any aggregate"))
        .count();
    assert!(
        aggregate_errors >= 2,
        "profile and capability modules must be flagged as unaggregated"
    );

    assert!(diag
        .repairs
        .iter()
        .any(|r| matches!(r.action, RepairAction::AddSchemaVersion { version: 1 })));
    assert!(diag
        .repairs
        .iter()
        .any(|r| matches!(r.action, RepairAction::AddAggregateImport { .. })));
}

#[test]
fn doctor_repairs_are_explained_with_confidence() {
    let repo_path = fixture_path();
    if !repo_path.exists() {
        return;
    }
    let repository = Repository::new(repo_path.display().to_string());
    let index = IndexService::new().build_static_only(&repository).unwrap();
    if index.static_index.has_parse_errors() {
        return;
    }

    let engine = ConfigurationEngine::new(repo_path.display().to_string());
    let diag = engine.diagnose(&index);

    assert!(!diag.repairs.is_empty());
    for repair in &diag.repairs {
        assert!(!repair.description.is_empty());
        assert!(repair.confidence > 0.0 && repair.confidence <= 1.0);
    }
}
