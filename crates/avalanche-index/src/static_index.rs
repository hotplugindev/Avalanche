use std::collections::HashMap;
use std::path::{Path, PathBuf};

use avalanche_model::module::{Module, ModuleId, ModuleKind};
use avalanche_model::scope::Scope;
use avalanche_source::{Assignment, ExtractedAssignment, SourceService};

use crate::error::{IndexError, IndexResult};

#[derive(Debug, Clone)]
pub struct StaticIndex {
    pub modules: Vec<Module>,
    pub assignments: HashMap<String, Vec<Assignment>>,
    pub imports: HashMap<String, Vec<String>>,
    pub source_locations: HashMap<String, Vec<avalanche_model::source::SourceLocation>>,
    pub parse_errors: Vec<ParseErrorRecord>,
    pub parsed_files: Vec<ParsedFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseErrorRecord {
    pub file: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub relative_path: String,
    pub absolute_path: PathBuf,
    pub module_kind: Option<ModuleKind>,
    pub scope: Option<Scope>,
    pub assignments: Vec<ExtractedAssignment>,
    pub import_paths: Vec<String>,
}

impl StaticIndex {
    pub fn empty() -> Self {
        Self {
            modules: Vec::new(),
            assignments: HashMap::new(),
            imports: HashMap::new(),
            source_locations: HashMap::new(),
            parse_errors: Vec::new(),
            parsed_files: Vec::new(),
        }
    }

    pub fn assignment_for(&self, option_path: &str) -> &[Assignment] {
        self.assignments
            .get(option_path)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn source_location_for(&self, option_path: &str) -> &[avalanche_model::source::SourceLocation] {
        self.source_locations
            .get(option_path)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn modules_of_kind(&self, kind: &ModuleKind) -> Vec<&Module> {
        self.modules.iter().filter(|m| &m.id.kind == kind).collect()
    }

    pub fn auto_editable_assignments(&self, option_path: &str) -> Vec<&Assignment> {
        self.assignment_for(option_path)
            .iter()
            .filter(|a| a.is_auto_editable())
            .collect()
    }

    pub fn has_parse_errors(&self) -> bool {
        !self.parse_errors.is_empty()
    }
}

pub fn build_static_index(repo_path: &Path, source: &SourceService) -> IndexResult<StaticIndex> {
    if !repo_path.exists() {
        return Err(IndexError::RepoNotFound {
            path: repo_path.display().to_string(),
        });
    }

    let nix_files = walk_nix_files(repo_path)?;
    let mut index = StaticIndex::empty();

    for abs_path in &nix_files {
        let relative = abs_path
            .strip_prefix(repo_path)
            .unwrap_or(abs_path)
            .display()
            .to_string();

        let file = match source.parse_path(abs_path) {
            Ok(f) => f,
            Err(e) => {
                index.parse_errors.push(ParseErrorRecord {
                    file: relative,
                    reason: e.to_string(),
                });
                continue;
            }
        };

        let assignments = source.extract_assignments(&file);
        let imports = source.find_imports(&file);
        let (module_kind, scope) = classify_module(&relative);

        let import_paths: Vec<String> = imports.iter().map(|i| i.path.clone()).collect();

        for a in &assignments {
            let dot_path = a.dot_path();
            let assignment = a.to_assignment(&relative);
            let location = assignment.location.clone();
            index
                .assignments
                .entry(dot_path.clone())
                .or_default()
                .push(assignment);
            index.source_locations.entry(dot_path).or_default().push(location);
        }

        if !import_paths.is_empty() {
            index.imports.insert(relative.clone(), import_paths.clone());
        }

        if let Some(ref kind) = module_kind {
            let module = Module {
                id: ModuleId {
                    kind: kind.clone(),
                    scope,
                    name: module_name(&relative),
                },
                path: relative.clone(),
                imported: true,
            };
            index.modules.push(module);
        }

        index.parsed_files.push(ParsedFile {
            relative_path: relative,
            absolute_path: abs_path.clone(),
            module_kind,
            scope,
            assignments,
            import_paths,
        });
    }

    Ok(index)
}

fn walk_nix_files(root: &Path) -> IndexResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_recursive(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> IndexResult<()> {
    let entries = std::fs::read_dir(dir).map_err(|e| IndexError::WalkFailed {
        path: dir.display().to_string(),
        reason: e.to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| IndexError::WalkFailed {
            path: dir.display().to_string(),
            reason: e.to_string(),
        })?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().map(|n| n.to_string_lossy().to_string());
            if matches!(
                dir_name.as_deref(),
                Some(".git") | Some("target") | Some("node_modules")
            ) {
                continue;
            }
            walk_recursive(&path, files)?;
        } else if path.extension().map(|e| e == "nix").unwrap_or(false) {
            files.push(path);
        }
    }

    Ok(())
}

fn classify_module(relative_path: &str) -> (Option<ModuleKind>, Option<Scope>) {
    let path = relative_path.replace('\\', "/");

    if path.starts_with("modules/capabilities/") {
        (Some(ModuleKind::Capability), extract_scope_from_path(&path))
    } else if path.starts_with("modules/programs/") {
        (Some(ModuleKind::Program), extract_scope_from_path(&path))
    } else if path.starts_with("modules/profiles/") {
        (Some(ModuleKind::Profile), extract_scope_from_path(&path))
    } else if path.starts_with("modules/desktop/") {
        (Some(ModuleKind::Desktop), extract_scope_from_path(&path))
    } else if path.starts_with("modules/aggregate/") {
        let scope = if path.contains("nixos") {
            Some(Scope::System)
        } else {
            Some(Scope::Home)
        };
        (Some(ModuleKind::Aggregate), scope)
    } else if path.starts_with("modules/core/") {
        (Some(ModuleKind::Core), None)
    } else {
        (None, None)
    }
}

fn extract_scope_from_path(path: &str) -> Option<Scope> {
    path.split('/').find_map(|part| match part {
        "system" => Some(Scope::System),
        "home" => Some(Scope::Home),
        _ => None,
    })
}

fn module_name(relative_path: &str) -> String {
    let path = relative_path.replace('\\', "/");
    path.strip_prefix("modules/")
        .unwrap_or(&path)
        .trim_end_matches(".nix")
        .trim_end_matches("/default")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_capability_system() {
        let (kind, scope) = classify_module("modules/capabilities/system/audio/pipewire.nix");
        assert_eq!(kind, Some(ModuleKind::Capability));
        assert_eq!(scope, Some(Scope::System));
    }

    #[test]
    fn classify_capability_home() {
        let (kind, scope) = classify_module("modules/capabilities/home/git/default.nix");
        assert_eq!(kind, Some(ModuleKind::Capability));
        assert_eq!(scope, Some(Scope::Home));
    }

    #[test]
    fn classify_program() {
        let (kind, scope) = classify_module("modules/programs/home/ai/codex.nix");
        assert_eq!(kind, Some(ModuleKind::Program));
        assert_eq!(scope, Some(Scope::Home));
    }

    #[test]
    fn classify_profile() {
        let (kind, scope) = classify_module("modules/profiles/system/workstation.nix");
        assert_eq!(kind, Some(ModuleKind::Profile));
        assert_eq!(scope, Some(Scope::System));
    }

    #[test]
    fn classify_desktop() {
        let (kind, scope) = classify_module("modules/desktop/mango/home/default.nix");
        assert_eq!(kind, Some(ModuleKind::Desktop));
        assert_eq!(scope, Some(Scope::Home));
    }

    #[test]
    fn classify_aggregates() {
        let (kind, scope) = classify_module("modules/aggregate/nixos.nix");
        assert_eq!(kind, Some(ModuleKind::Aggregate));
        assert_eq!(scope, Some(Scope::System));
        let (kind, scope) = classify_module("modules/aggregate/home.nix");
        assert_eq!(kind, Some(ModuleKind::Aggregate));
        assert_eq!(scope, Some(Scope::Home));
    }

    #[test]
    fn classify_core_and_hosts() {
        let (kind, _) = classify_module("modules/core/host/options.nix");
        assert_eq!(kind, Some(ModuleKind::Core));
        let (kind, _) = classify_module("hosts/pc/default.nix");
        assert_eq!(kind, None);
    }

    #[test]
    fn module_name_extraction() {
        assert_eq!(
            module_name("modules/capabilities/system/audio/pipewire.nix"),
            "capabilities/system/audio/pipewire"
        );
        assert_eq!(
            module_name("modules/capabilities/home/git/default.nix"),
            "capabilities/home/git"
        );
    }

    #[test]
    fn walk_skips_git_directory() {
        let dir = std::env::temp_dir().join("avalanche-index-test-walk");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::write(dir.join(".git/config.nix"), "{}").unwrap();
        std::fs::write(dir.join("real.nix"), "{ a = 1; }").unwrap();

        let files = walk_nix_files(&dir).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].ends_with("real.nix"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn build_static_index_on_fixture() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/simple-eval");
        if !fixture.exists() {
            return;
        }
        let source = SourceService::new();
        let index = build_static_index(&fixture, &source).unwrap();
        assert!(!index.parsed_files.is_empty());
        assert!(!index.assignments.is_empty());
        assert!(!index.has_parse_errors());
    }

    #[test]
    fn parse_errors_are_recorded_not_swallowed() {
        let dir = std::env::temp_dir().join("avalanche-index-test-broken");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("broken.nix"), "{ a = ;;").unwrap();
        std::fs::write(dir.join("fine.nix"), "{ b = 1; }").unwrap();

        let source = SourceService::new();
        let index = build_static_index(&dir, &source).unwrap();
        assert!(index.has_parse_errors());
        assert_eq!(index.parse_errors.len(), 1);
        assert!(index.parse_errors[0].file.contains("broken.nix"));
        assert!(index.assignments.contains_key("b"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
