use avalanche_git::{GitService, GitStatus};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_repo() -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = PathBuf::from("/tmp/opencode").join(format!(
        "avalanche-git-test-{}-{}",
        std::process::id(),
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    git(&dir, &["init", "-q"]);
    git(&dir, &["config", "user.email", "test@test.com"]);
    git(&dir, &["config", "user.name", "test"]);
    dir
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

fn write_file(dir: &Path, name: &str, content: &str) {
    fs::write(dir.join(name), content).unwrap();
}

fn service(dir: &Path) -> GitService {
    GitService::new(dir.to_str().unwrap())
}

#[test]
fn status_clean_after_commit() {
    let dir = temp_repo();
    let svc = service(&dir);
    write_file(&dir, "a.txt", "hello\n");
    git(&dir, &["add", "a.txt"]);
    git(&dir, &["commit", "-m", "initial"]);

    let st = svc.status().unwrap();
    assert!(st.branch.is_some());
    assert_eq!(st.ahead, 0);
    assert_eq!(st.behind, 0);
    assert!(st.staged.is_empty());
    assert!(st.modified.is_empty());
    assert!(st.untracked.is_empty());
    assert!(st.deleted.is_empty());
    assert!(st.conflicted.is_empty());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn status_separates_staged_modified_untracked() {
    let dir = temp_repo();
    let svc = service(&dir);
    write_file(&dir, "a.txt", "one\n");
    git(&dir, &["add", "a.txt"]);
    git(&dir, &["commit", "-m", "base"]);

    write_file(&dir, "b.txt", "two\n");
    git(&dir, &["add", "b.txt"]);
    write_file(&dir, "c.txt", "three\n");
    write_file(&dir, "a.txt", "one modified\n");

    let st = svc.status().unwrap();
    assert_eq!(st.staged, vec!["b.txt".to_string()]);
    assert_eq!(st.modified, vec!["a.txt".to_string()]);
    assert_eq!(st.untracked, vec!["c.txt".to_string()]);
    assert!(st.deleted.is_empty());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn status_detects_worktree_deletion() {
    let dir = temp_repo();
    let svc = service(&dir);
    write_file(&dir, "a.txt", "x\n");
    git(&dir, &["add", "a.txt"]);
    git(&dir, &["commit", "-m", "base"]);
    fs::remove_file(dir.join("a.txt")).unwrap();

    let st = svc.status().unwrap();
    assert_eq!(st.deleted, vec!["a.txt".to_string()]);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn history_lists_commits_newest_first() {
    let dir = temp_repo();
    let svc = service(&dir);
    write_file(&dir, "a.txt", "1\n");
    git(&dir, &["add", "a.txt"]);
    git(&dir, &["commit", "-m", "first"]);
    write_file(&dir, "a.txt", "2\n");
    git(&dir, &["add", "a.txt"]);
    git(&dir, &["commit", "-m", "second"]);

    let hist = svc.history(10).unwrap();
    assert_eq!(hist.len(), 2);
    assert_eq!(hist[0].message, "second");
    assert_eq!(hist[1].message, "first");
    assert_eq!(hist[0].hash.len(), 40);
    assert_eq!(hist[0].short_hash.len(), 7);
    assert_eq!(hist[0].author, "test");
    assert!(hist[0].timestamp > 0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn history_respects_max_count() {
    let dir = temp_repo();
    let svc = service(&dir);
    for i in 0..3 {
        write_file(&dir, "a.txt", &format!("{i}\n"));
        git(&dir, &["add", "a.txt"]);
        git(&dir, &["commit", "-m", &format!("c{i}")]);
    }
    let hist = svc.history(2).unwrap();
    assert_eq!(hist.len(), 2);
    assert_eq!(hist[0].message, "c2");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn branches_create_checkout_current() {
    let dir = temp_repo();
    let svc = service(&dir);
    write_file(&dir, "a.txt", "x\n");
    git(&dir, &["add", "a.txt"]);
    git(&dir, &["commit", "-m", "base"]);

    let start = svc.current_branch().unwrap().unwrap();
    svc.create_branch("feature").unwrap();
    assert_eq!(svc.current_branch().unwrap().as_deref(), Some("feature"));

    let names: Vec<String> = svc
        .branches()
        .unwrap()
        .into_iter()
        .map(|b| b.name)
        .collect();
    assert!(names.contains(&start));
    assert!(names.contains(&"feature".to_string()));

    svc.checkout_branch(&start).unwrap();
    assert_eq!(
        svc.current_branch().unwrap().as_deref(),
        Some(start.as_str())
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn commit_returns_head_hash() {
    let dir = temp_repo();
    let svc = service(&dir);
    write_file(&dir, "a.txt", "x\n");

    let hash = svc.commit("add a").unwrap();
    assert_eq!(hash.len(), 40);
    assert_eq!(svc.head().unwrap(), hash);
    assert!(!svc.is_dirty().unwrap());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn rollback_restores_working_tree() {
    let dir = temp_repo();
    let svc = service(&dir);
    write_file(&dir, "a.txt", "original\n");
    git(&dir, &["add", "a.txt"]);
    git(&dir, &["commit", "-m", "base"]);

    write_file(&dir, "a.txt", "tampered\n");
    svc.rollback(&["a.txt".to_string()]).unwrap();

    let content = fs::read_to_string(dir.join("a.txt")).unwrap();
    assert_eq!(content, "original\n");
    assert!(!svc.is_dirty().unwrap());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn conflicts_reports_merge_stages() {
    let dir = temp_repo();
    let svc = service(&dir);
    write_file(&dir, "f.txt", "base\n");
    git(&dir, &["add", "f.txt"]);
    git(&dir, &["commit", "-m", "base"]);
    let main = svc.current_branch().unwrap().unwrap();

    git(&dir, &["checkout", "-b", "side"]);
    write_file(&dir, "f.txt", "side\n");
    git(&dir, &["add", "f.txt"]);
    git(&dir, &["commit", "-m", "side"]);

    git(&dir, &["checkout", main.as_str()]);
    write_file(&dir, "f.txt", "ours\n");
    git(&dir, &["add", "f.txt"]);
    git(&dir, &["commit", "-m", "ours"]);

    let _ = Command::new("git")
        .args(["merge", "side"])
        .current_dir(&dir)
        .output();

    let conflicts = svc.conflicts().unwrap();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].path, "f.txt");
    assert_eq!(conflicts[0].base.as_deref(), Some("base\n"));
    assert_eq!(conflicts[0].ours.as_deref(), Some("ours\n"));
    assert_eq!(conflicts[0].theirs.as_deref(), Some("side\n"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn pull_push_without_remote_error_gracefully() {
    let dir = temp_repo();
    let svc = service(&dir);
    write_file(&dir, "a.txt", "x\n");
    git(&dir, &["add", "a.txt"]);
    git(&dir, &["commit", "-m", "base"]);

    assert!(svc.pull().is_err());
    assert!(svc.push().is_err());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn diff_shows_worktree_changes() {
    let dir = temp_repo();
    let svc = service(&dir);
    write_file(&dir, "a.txt", "one\n");
    git(&dir, &["add", "a.txt"]);
    git(&dir, &["commit", "-m", "base"]);
    write_file(&dir, "a.txt", "two\n");

    let d = svc.diff().unwrap();
    assert!(d.contains("-one"));
    assert!(d.contains("+two"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn status_serializes_roundtrip() {
    let st = GitStatus {
        branch: Some("main".to_string()),
        ahead: 1,
        behind: 2,
        staged: vec!["a.txt".to_string()],
        modified: vec!["b.txt".to_string()],
        untracked: vec!["c.txt".to_string()],
        deleted: vec![],
        conflicted: vec![],
    };
    let json = serde_json::to_string(&st).unwrap();
    let back: GitStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(st, back);
}
