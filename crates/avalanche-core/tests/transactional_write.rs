use std::path::{Path, PathBuf};

use avalanche_core::ConfigurationEngine;
use avalanche_model::{MutationIntent, Profile, ProfileId, Repository, Scope, TransactionStatus};
use avalanche_write::WriteError;

fn write_file(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn read_disk(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

fn fixture(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("avalanche-txn-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    write_file(
        &dir,
        "modules/capabilities/system/audio/pipewire.nix",
        "{\n  services.pipewire.enable = true;\n}\n",
    );
    write_file(
        &dir,
        "modules/profiles/system/workstation.nix",
        "{\n  networking.hostName = \"base\";\n}\n",
    );
    write_file(
        &dir,
        "hosts/pc/default.nix",
        "{\n  networking.hostName = \"pc-host\";\n  services.foo.enable = true;\n}\n",
    );
    dir
}

fn repo_with_profile(root: &Path) -> Repository {
    let mut r = Repository::new(root.display().to_string());
    r.profiles.push(Profile {
        id: ProfileId::new(Scope::System, "workstation"),
        source_file: "modules/profiles/system/workstation.nix".to_string(),
    });
    r
}

fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn execute_set_option_writes_to_disk_and_returns_applied_transaction() {
    let dir = fixture("apply");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

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
    assert!(txn.diff.is_some());
    assert!(txn.validation.map(|v| v.passed).unwrap_or(false));
    assert_eq!(txn.file_changes.len(), 1);

    let on_disk = read_disk(&dir, "modules/capabilities/system/audio/pipewire.nix");
    assert!(on_disk.contains("services.pipewire.enable = false;"));
    assert!(!on_disk.contains("= true;"));

    cleanup(&dir);
}

#[test]
fn execute_reset_host_override_deletes_line_and_preserves_sibling() {
    let dir = fixture("reset");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let txn = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::ResetHostOverride {
                path: "networking.hostName".to_string(),
                host: "pc".to_string(),
            }],
        )
        .unwrap();

    assert_eq!(txn.status, TransactionStatus::Applied);
    let on_disk = read_disk(&dir, "hosts/pc/default.nix");
    assert!(!on_disk.contains("networking.hostName"));
    assert!(!on_disk.contains("= false"));
    assert!(on_disk.contains("services.foo.enable = true;"));

    cleanup(&dir);
}

#[test]
fn execute_set_profile_default_renders_mk_default_on_disk() {
    let dir = fixture("mkdefault");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let txn = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::SetProfileDefault {
                path: "networking.hostName".to_string(),
                value: serde_json::json!("pc"),
                profile: "workstation".to_string(),
            }],
        )
        .unwrap();

    assert_eq!(txn.status, TransactionStatus::Applied);
    let on_disk = read_disk(&dir, "modules/profiles/system/workstation.nix");
    assert!(on_disk.contains("lib.mkDefault"));
    assert!(on_disk.contains("\"pc\""));

    cleanup(&dir);
}

#[test]
fn execute_is_idempotent_when_value_already_set() {
    let dir = fixture("idem");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());

    let index = engine.index_static(&repo).unwrap();
    engine
        .execute_mutation(
            &index,
            vec![MutationIntent::SetOption {
                path: "services.pipewire.enable".to_string(),
                value: serde_json::json!(false),
                scope: "system".to_string(),
            }],
        )
        .unwrap();

    let index2 = engine.index_static(&repo).unwrap();
    let txn2 = engine
        .execute_mutation(
            &index2,
            vec![MutationIntent::SetOption {
                path: "services.pipewire.enable".to_string(),
                value: serde_json::json!(false),
                scope: "system".to_string(),
            }],
        )
        .unwrap();

    assert_eq!(txn2.status, TransactionStatus::Applied);
    assert!(txn2.file_changes.is_empty());
    let on_disk = read_disk(&dir, "modules/capabilities/system/audio/pipewire.nix");
    assert!(on_disk.contains("services.pipewire.enable = false;"));

    cleanup(&dir);
}

#[test]
fn execute_module_without_aggregate_fails_closed_and_leaves_disk_untouched() {
    let dir = fixture("unsup");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let before = read_disk(&dir, "modules/capabilities/system/audio/pipewire.nix");
    let err = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::CreateModule {
                name: "test".to_string(),
                kind: "program".to_string(),
            }],
        )
        .unwrap_err();
    assert!(matches!(err, WriteError::EditFailed(_)));

    let after = read_disk(&dir, "modules/capabilities/system/audio/pipewire.nix");
    assert_eq!(before, after);
    assert!(!dir.join("modules/programs/system/test.nix").exists());

    cleanup(&dir);
}

#[test]
fn execute_stale_index_is_rejected_before_writing() {
    let dir = fixture("stale");
    let repo = repo_with_profile(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let mut index = engine.index_static(&repo).unwrap();
    index.fingerprint.git_head = Some("stale-head".to_string());

    let before = read_disk(&dir, "modules/capabilities/system/audio/pipewire.nix");
    let err = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::SetOption {
                path: "services.pipewire.enable".to_string(),
                value: serde_json::json!(false),
                scope: "system".to_string(),
            }],
        )
        .unwrap_err();
    assert!(matches!(err, WriteError::StaleIndex));

    let after = read_disk(&dir, "modules/capabilities/system/audio/pipewire.nix");
    assert_eq!(before, after);

    cleanup(&dir);
}
