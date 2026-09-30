use std::path::Path;
use std::process::Command;

use avalanche_model::repository::Fingerprint;

use crate::error::{IndexError, IndexResult};

pub fn compute_fingerprint(repo_path: &Path) -> IndexResult<Fingerprint> {
    let git_head = git_head(repo_path);
    let working_tree_hash = working_tree_hash(repo_path);
    let flake_lock_hash = flake_lock_hash(repo_path);
    let schema_version = detect_schema_version(repo_path);

    Ok(Fingerprint {
        git_head,
        working_tree_hash,
        flake_lock_hash,
        schema_version,
        index_schema_version: INDEX_SCHEMA_VERSION,
    })
}

pub const INDEX_SCHEMA_VERSION: u32 = 1;

fn git_head(repo_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo_path)
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn working_tree_hash(repo_path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let porcelain = String::from_utf8_lossy(&output.stdout).to_string();
    if porcelain.trim().is_empty() {
        Some("clean".to_string())
    } else {
        let hash = simple_hash(porcelain.as_bytes());
        Some(hash)
    }
}

fn flake_lock_hash(repo_path: &Path) -> Option<String> {
    let lock_path = repo_path.join("flake.lock");
    let content = std::fs::read(&lock_path).ok()?;
    Some(simple_hash(&content))
}

fn detect_schema_version(repo_path: &Path) -> Option<u32> {
    let schema_file = repo_path.join("modules/core/schema.nix");
    if let Ok(content) = std::fs::read_to_string(&schema_file) {
        if content.contains("schemaVersion = 1") || content.contains("schemaVersion = 1;") {
            return Some(1);
        }
    }
    let flake = repo_path.join("flake.nix");
    if let Ok(content) = std::fs::read_to_string(&flake) {
        if content.contains("gb.schemaVersion") || content.contains("schemaVersion") {
            return Some(1);
        }
    }
    None
}

fn simple_hash(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn is_stale(stored: &Fingerprint, current: &Fingerprint) -> bool {
    stored != current
}

pub fn validate_fingerprint(fingerprint: &Fingerprint) -> IndexResult<()> {
    if fingerprint.git_head.is_none() {
        return Err(IndexError::FingerprintFailed {
            reason: "could not determine git HEAD".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_hash_deterministic() {
        let h1 = simple_hash(b"hello");
        let h2 = simple_hash(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn simple_hash_differs() {
        let h1 = simple_hash(b"hello");
        let h2 = simple_hash(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn index_schema_version_is_one() {
        assert_eq!(INDEX_SCHEMA_VERSION, 1);
    }

    #[test]
    fn stale_detection_same_fingerprint() {
        let fp = Fingerprint {
            git_head: Some("abc".into()),
            working_tree_hash: Some("clean".into()),
            flake_lock_hash: Some("xyz".into()),
            schema_version: Some(1),
            index_schema_version: 1,
        };
        assert!(!is_stale(&fp, &fp));
    }

    #[test]
    fn stale_detection_different_head() {
        let fp1 = Fingerprint {
            git_head: Some("abc".into()),
            working_tree_hash: Some("clean".into()),
            flake_lock_hash: Some("xyz".into()),
            schema_version: Some(1),
            index_schema_version: 1,
        };
        let fp2 = Fingerprint {
            git_head: Some("def".into()),
            ..fp1.clone()
        };
        assert!(is_stale(&fp1, &fp2));
    }

    #[test]
    fn stale_detection_different_tree() {
        let fp1 = Fingerprint {
            git_head: Some("abc".into()),
            working_tree_hash: Some("clean".into()),
            flake_lock_hash: Some("xyz".into()),
            schema_version: Some(1),
            index_schema_version: 1,
        };
        let fp2 = Fingerprint {
            working_tree_hash: Some("dirty_hash".into()),
            ..fp1.clone()
        };
        assert!(is_stale(&fp1, &fp2));
    }

    #[test]
    fn compute_fingerprint_on_real_repo() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../example");
        if !repo.exists() {
            return;
        }
        let fp = compute_fingerprint(&repo).unwrap();
        assert!(fp.git_head.is_some());
        assert_eq!(fp.index_schema_version, INDEX_SCHEMA_VERSION);
    }
}
