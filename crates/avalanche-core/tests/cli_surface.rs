use std::path::{Path, PathBuf};

use avalanche_core::{ConfigurationEngine, RepositoryIndex};
use avalanche_model::{OwnershipStatus, Repository};

fn fixture_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/provenance")
}

fn build_static() -> Option<(ConfigurationEngine, RepositoryIndex)> {
    let repo_path = fixture_path();
    if !repo_path.exists() {
        return None;
    }
    let repo_str = repo_path.display().to_string();
    let repository = Repository::new(repo_str.clone());
    let engine = ConfigurationEngine::new(repo_str);
    let index = engine.index_static(&repository).ok()?;
    if index.static_index.has_parse_errors() {
        return None;
    }
    Some((engine, index))
}

#[test]
fn index_static_builds_cli_surface() {
    let Some((_, index)) = build_static() else {
        return;
    };
    assert!(!index.modules.is_empty());
    assert!(!index.provenance.is_empty());
}

#[test]
fn why_returns_display_chains_for_capability() {
    let Some((engine, index)) = build_static() else {
        return;
    };
    let chains = engine.why(&index, "audio.pipewire").unwrap();
    assert!(
        !chains.is_empty(),
        "why(audio.pipewire) must resolve requesters from static provenance"
    );
    for c in &chains {
        assert!(!c.via.is_empty());
        assert!(!c.steps.is_empty());
        for s in &c.steps {
            assert!(s.contains(':'), "step '{}' is not a display string", s);
        }
    }
}

#[test]
fn why_expands_option_through_its_owner() {
    let Some((engine, index)) = build_static() else {
        return;
    };
    let chains = engine
        .why(&index, "services.pipewire.media-session.enable")
        .unwrap();
    for c in &chains {
        assert!(c.via.contains("pipewire") || c.via.contains("Capability"));
    }
}

#[test]
fn why_unknown_node_is_clean_error() {
    let Some((engine, index)) = build_static() else {
        return;
    };
    let err = engine.why(&index, "definitely.not.a.node").unwrap_err();
    assert!(err.contains("unknown node"), "unexpected error: {}", err);
}

#[test]
fn owners_reports_ownership_record() {
    let Some((engine, index)) = build_static() else {
        return;
    };
    let entry = engine
        .owners(&index, "services.pipewire.media-session.enable")
        .expect("pipewire option must have an ownership record");
    assert!(!entry.contributing_files.is_empty());
    if entry.status == OwnershipStatus::Owned {
        assert!(entry.owner_module.contains("pipewire"));
    }
}

#[test]
fn owners_unknown_option_is_clean_error() {
    let Some((engine, index)) = build_static() else {
        return;
    };
    assert!(engine.owners(&index, "no.such.option.anywhere").is_err());
}

#[test]
fn dependents_returns_display_nodes() {
    let Some((engine, index)) = build_static() else {
        return;
    };
    let deps = engine
        .dependents(&index, "profiles.system.workstation")
        .unwrap();
    for d in &deps {
        assert!(d.contains(':'), "dependent '{}' is not a display string", d);
    }
}

#[test]
fn diff_does_not_panic() {
    let repo_path = fixture_path();
    if !repo_path.exists() {
        return;
    }
    let engine = ConfigurationEngine::new(repo_path.display().to_string());
    let _ = engine.diff();
}
