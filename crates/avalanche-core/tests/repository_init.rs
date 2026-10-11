use std::path::{Path, PathBuf};

use avalanche_core::ConfigurationEngine;
use avalanche_model::Repository;
use avalanche_write::{InitSpec, WriteError};

fn tmp(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("avalanche-init-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn spec(host: &str, user: &str) -> InitSpec {
    InitSpec {
        host_name: host.to_string(),
        host_system: "x86_64-linux".to_string(),
        user_name: user.to_string(),
        user_full_name: "Test User".to_string(),
        user_email: "test@example.com".to_string(),
    }
}

fn exists(dir: &Path, rel: &str) -> bool {
    dir.join(rel).exists()
}

#[test]
fn init_creates_valid_repository() {
    let dir = tmp("valid");
    ConfigurationEngine::init_repository(&dir, &spec("pc", "alice")).unwrap();

    assert!(exists(&dir, "flake.nix"));
    assert!(exists(&dir, "hosts/pc/default.nix"));
    assert!(exists(&dir, "hosts/default.nix"));
    assert!(exists(&dir, "users/alice.nix"));
    assert!(exists(&dir, "users/default.nix"));
    assert!(exists(&dir, "modules/core/schema.nix"));
    assert!(exists(&dir, "modules/aggregate/nixos.nix"));
    assert!(exists(&dir, "modules/aggregate/home.nix"));
    assert!(exists(&dir, ".git"));

    let engine = ConfigurationEngine::new(dir.display().to_string());
    let repo = Repository::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();
    let diag = engine.diagnose(&index);
    assert_eq!(
        diag.error_count(),
        0,
        "doctor reported errors: {:?}",
        diag.findings
            .iter()
            .filter(|f| f.severity == avalanche_model::ValidationSeverity::Error)
            .map(|f| f.message.clone())
            .collect::<Vec<_>>()
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn init_idempotent_adds_second_host() {
    let dir = tmp("idem");
    ConfigurationEngine::init_repository(&dir, &spec("pc", "alice")).unwrap();

    ConfigurationEngine::init_repository(&dir, &spec("laptop", "alice")).unwrap();

    assert!(exists(&dir, "hosts/pc/default.nix"));
    assert!(exists(&dir, "hosts/laptop/default.nix"));
    assert!(exists(&dir, "users/alice.nix"));

    let pc_content = std::fs::read_to_string(dir.join("hosts/pc/default.nix")).unwrap();
    assert!(pc_content.contains("\"pc\""));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn init_rejects_bad_names() {
    let dir = tmp("bad");
    let err = ConfigurationEngine::init_repository(&dir, &spec("1bad", "alice")).unwrap_err();
    assert!(matches!(err, WriteError::EditFailed(_)));
    assert!(!exists(&dir, "flake.nix"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn init_declares_schema_version_in_fingerprint() {
    let dir = tmp("schema");
    ConfigurationEngine::init_repository(&dir, &spec("pc", "alice")).unwrap();

    let engine = ConfigurationEngine::new(dir.display().to_string());
    let repo = Repository::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();
    assert_eq!(index.repository.schema_version, Some(1));

    let _ = std::fs::remove_dir_all(&dir);
}
