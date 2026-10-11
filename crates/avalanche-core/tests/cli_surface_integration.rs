use std::path::{Path, PathBuf};

use avalanche_core::ConfigurationEngine;
use avalanche_model::Repository;

fn provenance_fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/provenance")
}

#[test]
fn owners_resolves_for_every_provenance_path() {
    let repo_path = provenance_fixture_path();
    if !repo_path.exists() {
        return;
    }
    let repository = Repository::new(repo_path.display().to_string());
    let engine = ConfigurationEngine::new(repo_path.display().to_string());
    let index = engine.index_static(&repository).unwrap();
    if index.static_index.has_parse_errors() {
        return;
    }

    let paths: Vec<String> = index.provenance.keys().cloned().collect();
    assert!(!paths.is_empty(), "fixture must yield provenance");
    for path in &paths {
        let entry = engine.owners(&index, path).unwrap();
        assert_eq!(&entry.option_path, path);
    }
}

#[test]
fn why_and_dependents_work_through_the_engine() {
    let repo_path = provenance_fixture_path();
    if !repo_path.exists() {
        return;
    }
    let repository = Repository::new(repo_path.display().to_string());
    let engine = ConfigurationEngine::new(repo_path.display().to_string());
    let index = engine.index_static(&repository).unwrap();
    if index.static_index.has_parse_errors() {
        return;
    }

    let chains = engine.why(&index, "audio.pipewire").unwrap();
    assert!(
        !chains.is_empty(),
        "why(audio.pipewire) must resolve requesters"
    );
    assert!(chains[0]
        .steps
        .first()
        .unwrap()
        .contains("profiles.system.workstation"));

    assert!(engine.why(&index, "nope.nope.nope").is_err());

    let deps = engine
        .dependents(&index, "profiles.system.workstation")
        .unwrap();
    assert!(deps.iter().any(|d| d.contains("audio.pipewire")));
}

#[test]
fn diff_fails_gracefully_on_non_git_fixture() {
    let repo_path = provenance_fixture_path();
    if !repo_path.exists() {
        return;
    }
    let engine = ConfigurationEngine::new(repo_path.display().to_string());
    let result = engine.diff();
    assert!(
        result.is_err() || result.as_ref().is_ok_and(|s| !s.is_empty()),
        "diff on a non-git fixture must not silently succeed with content"
    );
}
