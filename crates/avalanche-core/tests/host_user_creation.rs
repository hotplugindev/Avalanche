use std::path::{Path, PathBuf};

use avalanche_core::ConfigurationEngine;
use avalanche_model::{MutationIntent, Repository};
use avalanche_write::WriteError;

fn write_file(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn read_file(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

fn fixture(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("avalanche-hu-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    write_file(
        &dir,
        "hosts/default.nix",
        "{\n  pc = {\n    system = \"x86_64-linux\";\n  };\n}\n",
    );
    write_file(
        &dir,
        "users/default.nix",
        "{\n  hotplugin = {\n    username = \"hotplugin\";\n  };\n}\n",
    );
    dir
}

fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn create_host_writes_default_nix_and_registers() {
    let dir = fixture("host-ok");
    let repo = Repository::new(dir.display().to_string());
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let plan = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::CreateHost {
                name: "laptop".to_string(),
            }],
        )
        .unwrap();

    assert_eq!(plan.create.len(), 1);
    assert_eq!(plan.create[0].path, "hosts/laptop/default.nix");
    assert!(plan.create[0]
        .content
        .contains("gb.host.name = \"laptop\";"));

    assert_eq!(plan.file_changes.len(), 1);
    assert_eq!(plan.file_changes[0].path, "hosts/default.nix");
    assert!(plan.file_changes[0].after.contains("laptop = {"));
    assert!(plan.file_changes[0]
        .after
        .contains("system = \"x86_64-linux\";"));

    let txn = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::CreateHost {
                name: "laptop".to_string(),
            }],
        )
        .unwrap();
    assert!(txn
        .diff
        .as_deref()
        .unwrap()
        .contains("hosts/laptop/default.nix"));

    let on_disk = read_file(&dir, "hosts/laptop/default.nix");
    assert!(on_disk.contains("gb.host.name = \"laptop\";"));
    let reg = read_file(&dir, "hosts/default.nix");
    assert!(reg.contains("laptop = {"));
    assert!(reg.contains("pc = {"));

    cleanup(&dir);
}

#[test]
fn create_host_duplicate_is_error() {
    let dir = fixture("host-dup");
    write_file(
        &dir,
        "hosts/pc/default.nix",
        "{\n  gb.host.name = \"pc\";\n}\n",
    );
    let repo = Repository::new(dir.display().to_string());
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let before = read_file(&dir, "hosts/default.nix");
    let err = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::CreateHost {
                name: "pc".to_string(),
            }],
        )
        .unwrap_err();

    assert!(matches!(err, WriteError::EditFailed(_)));
    assert!(err.to_string().contains("already exists"));
    assert_eq!(read_file(&dir, "hosts/default.nix"), before);

    cleanup(&dir);
}

#[test]
fn create_user_writes_identity_and_registers() {
    let dir = fixture("user-ok");
    let repo = Repository::new(dir.display().to_string());
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let plan = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::CreateUser {
                name: "bob".to_string(),
            }],
        )
        .unwrap();

    assert_eq!(plan.create.len(), 1);
    assert_eq!(plan.create[0].path, "users/bob.nix");
    assert!(plan.create[0].content.contains("home.username = \"bob\";"));
    assert!(plan.create[0]
        .content
        .contains("home.homeDirectory = \"/home/bob\";"));

    assert_eq!(plan.file_changes.len(), 1);
    assert_eq!(plan.file_changes[0].path, "users/default.nix");
    let after = &plan.file_changes[0].after;
    assert!(after.contains("bob = {"));
    assert!(after.contains("username = \"bob\";"));
    assert!(after.contains("homeModule = ./bob.nix;"));

    engine
        .execute_mutation(
            &index,
            vec![MutationIntent::CreateUser {
                name: "bob".to_string(),
            }],
        )
        .unwrap();
    let on_disk = read_file(&dir, "users/bob.nix");
    assert!(on_disk.contains("home.username = \"bob\";"));

    cleanup(&dir);
}

#[test]
fn create_user_duplicate_is_error() {
    let dir = fixture("user-dup");
    write_file(&dir, "users/bob.nix", "{\n  home.username = \"bob\";\n}\n");
    let repo = Repository::new(dir.display().to_string());
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let before = read_file(&dir, "users/default.nix");
    let err = engine
        .plan_mutation(
            &index,
            vec![MutationIntent::CreateUser {
                name: "bob".to_string(),
            }],
        )
        .unwrap_err();

    assert!(matches!(err, WriteError::EditFailed(_)));
    assert!(err.to_string().contains("already exists"));
    assert_eq!(read_file(&dir, "users/default.nix"), before);

    cleanup(&dir);
}

#[test]
fn create_host_leaves_repo_unchanged_on_validation_failure() {
    let dir = fixture("host-rollback");
    write_file(&dir, "hosts/laptop", "not-a-directory\n");
    let repo = Repository::new(dir.display().to_string());
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&repo).unwrap();

    let before = read_file(&dir, "hosts/default.nix");
    let err = engine
        .execute_mutation(
            &index,
            vec![MutationIntent::CreateHost {
                name: "laptop".to_string(),
            }],
        )
        .unwrap_err();

    assert!(matches!(err, WriteError::Io { .. }));
    assert_eq!(read_file(&dir, "hosts/default.nix"), before);
    assert_eq!(read_file(&dir, "hosts/laptop"), "not-a-directory\n");

    cleanup(&dir);
}
