use std::path::{Path, PathBuf};

use avalanche_core::ConfigurationEngine;
use avalanche_model::{Application, Repository, Scope, TransactionStatus};

fn write_file(root: &Path, rel: &str, content: &str) {
    let p = root.join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, content).unwrap();
}

fn read_disk(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

fn fixture(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("avalanche-appmgr-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    write_file(
        &dir,
        "modules/programs/system/steam.nix",
        "{\n  options.gb.programs.system.steam.enable = lib.mkEnableOption \"Steam\";\n}\n",
    );
    write_file(
        &dir,
        "modules/programs/home/web/firefox.nix",
        "{\n  options.gb.home.programs.web.firefox.enable = lib.mkEnableOption \"Firefox\";\n}\n",
    );
    write_file(
        &dir,
        "hosts/pc/default.nix",
        "{\n  networking.hostName = \"pc\";\n}\n",
    );
    dir
}

fn repo(root: &Path) -> Repository {
    Repository::new(root.display().to_string())
}

fn find_app<'a>(apps: &'a [Application], name: &str) -> &'a Application {
    apps.iter()
        .find(|a| a.id.name == name)
        .expect("app discovered")
}

#[test]
fn index_discovers_applications() {
    let dir = fixture("discover");
    let r = repo(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&r).unwrap();

    assert_eq!(index.applications.len(), 2);
    let steam = find_app(&index.applications, "steam");
    assert_eq!(steam.id.scope, Scope::System);
    assert_eq!(steam.module_path, "modules/programs/system/steam.nix");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn search_matches_name_category_and_requester_case_insensitively() {
    let dir = fixture("search");
    let r = repo(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());
    let index = engine.index_static(&r).unwrap();

    assert_eq!(engine.search_applications(&index, "STEAM").len(), 1);
    assert_eq!(engine.search_applications(&index, "web").len(), 1);
    assert_eq!(
        engine
            .search_applications(&index, "programs.home.web")
            .len(),
        1
    );
    assert_eq!(engine.search_applications(&index, "").len(), 2);
    assert!(engine.search_applications(&index, "nonexistent").is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn enable_writes_host_override_and_disable_deletes_it() {
    let dir = fixture("enable");
    let r = repo(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());

    let index = engine.index_static(&r).unwrap();
    let steam = find_app(&index.applications, "steam").clone();

    let txn = engine
        .set_application_enabled(&index, &steam, "pc", true)
        .unwrap();
    assert_eq!(txn.status, TransactionStatus::Applied);
    let host = read_disk(&dir, "hosts/pc/default.nix");
    assert!(host.contains("gb.programs.system.steam.enable = true;"));

    let index2 = engine.index_static(&r).unwrap();
    let steam2 = find_app(&index2.applications, "steam").clone();
    let txn2 = engine
        .set_application_enabled(&index2, &steam2, "pc", false)
        .unwrap();
    assert_eq!(txn2.status, TransactionStatus::Applied);
    let host2 = read_disk(&dir, "hosts/pc/default.nix");
    assert!(!host2.contains("gb.programs.system.steam.enable"));
    assert!(!host2.contains("= false"));
    assert!(host2.contains("networking.hostName"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn enable_is_idempotent_and_round_trips_option_path_for_flat_system_apps() {
    let dir = fixture("idem");
    let r = repo(&dir);
    let engine = ConfigurationEngine::new(dir.display().to_string());

    let index = engine.index_static(&r).unwrap();
    let steam = find_app(&index.applications, "steam").clone();
    assert_eq!(steam.id.option_path(), "gb.programs.system.steam");

    engine
        .set_application_enabled(&index, &steam, "pc", true)
        .unwrap();

    let index2 = engine.index_static(&r).unwrap();
    let steam2 = find_app(&index2.applications, "steam").clone();
    let txn = engine
        .set_application_enabled(&index2, &steam2, "pc", true)
        .unwrap();
    assert!(txn.file_changes.is_empty());

    let _ = std::fs::remove_dir_all(&dir);
}
