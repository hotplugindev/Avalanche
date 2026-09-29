use std::path::{Path, PathBuf};

use avalanche_model::source::{Position, SourceSpan};

use crate::ast::{Expr, ImportRef, NixFile};
use crate::parser::{parse, ParseError};

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedImport {
    pub raw_path: String,
    pub resolved_path: PathBuf,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportGraph {
    pub root: PathBuf,
    pub nodes: Vec<ImportNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportNode {
    pub file: PathBuf,
    pub imports: Vec<ResolvedImport>,
}

pub fn extract_imports(file: &NixFile) -> Vec<ImportRef> {
    file.imports.clone()
}

pub fn resolve_import(base_dir: &Path, import_path: &str) -> Option<PathBuf> {
    if import_path.starts_with('<') && import_path.ends_with('>') {
        return None;
    }

    let candidate = if import_path.starts_with('/') {
        PathBuf::from(import_path)
    } else {
        base_dir.join(import_path)
    };

    let normalized = normalize_path(&candidate);

    if normalized.is_file() {
        return Some(normalized);
    }

    if normalized.is_dir() {
        let default_nix = normalized.join("default.nix");
        if default_nix.is_file() {
            return Some(default_nix);
        }
    }

    let with_nix = PathBuf::from(format!("{}.nix", normalized.display()));
    if with_nix.is_file() {
        return Some(with_nix);
    }

    None
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            other => {
                components.push(other);
            }
        }
    }
    components.iter().collect()
}

pub fn build_import_graph(root: &Path, max_depth: usize) -> Result<ImportGraph, ParseError> {
    let mut nodes = Vec::new();
    let mut visited = std::collections::HashSet::new();
    build_graph_recursive(root, &mut nodes, &mut visited, 0, max_depth)?;
    Ok(ImportGraph {
        root: root.to_path_buf(),
        nodes,
    })
}

fn build_graph_recursive(
    file: &Path,
    nodes: &mut Vec<ImportNode>,
    visited: &mut std::collections::HashSet<PathBuf>,
    depth: usize,
    max_depth: usize,
) -> Result<(), ParseError> {
    let canonical = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
    if visited.contains(&canonical) || depth > max_depth {
        return Ok(());
    }
    visited.insert(canonical.clone());

    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };

    let nix_file = parse(&content)?;
    let base_dir = file.parent().unwrap_or(Path::new("."));

    let mut resolved_imports = Vec::new();
    for import_ref in &nix_file.imports {
        if let Some(resolved) = resolve_import(base_dir, &import_ref.path) {
            resolved_imports.push(ResolvedImport {
                raw_path: import_ref.path.clone(),
                resolved_path: resolved.clone(),
                span: import_ref.span,
            });
            build_graph_recursive(&resolved, nodes, visited, depth + 1, max_depth)?;
        }
    }

    nodes.push(ImportNode {
        file: canonical,
        imports: resolved_imports,
    });

    Ok(())
}

pub fn find_import_sites(file: &NixFile) -> Vec<ImportSite> {
    let mut sites = Vec::new();
    find_import_sites_recursive(&file.expr, &mut sites);
    sites
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImportSite {
    pub path: String,
    pub span: SourceSpan,
    pub context: ImportContext,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportContext {
    TopLevel,
    InList,
    InLetBinding,
    InAttrValue,
}

fn find_import_sites_recursive(expr: &Expr, sites: &mut Vec<ImportSite>) {
    match expr {
        Expr::Apply { func, arg } => {
            if let Expr::Ident(name) = func.as_ref() {
                if name == "import" {
                    if let Expr::Path(p) = arg.as_ref() {
                        sites.push(ImportSite {
                            path: p.clone(),
                            span: SourceSpan::new(
                                Position { line: 0, column: 0 },
                                Position { line: 0, column: 0 },
                            ),
                            context: ImportContext::TopLevel,
                        });
                    }
                }
            }
            find_import_sites_recursive(func, sites);
            find_import_sites_recursive(arg, sites);
        }
        Expr::List(items) => {
            for item in items {
                if let Expr::Apply { func, arg } = item {
                    if let Expr::Ident(name) = func.as_ref() {
                        if name == "import" {
                            if let Expr::Path(p) = arg.as_ref() {
                                sites.push(ImportSite {
                                    path: p.clone(),
                                    span: SourceSpan::new(
                                        Position { line: 0, column: 0 },
                                        Position { line: 0, column: 0 },
                                    ),
                                    context: ImportContext::InList,
                                });
                                continue;
                            }
                        }
                    }
                }
                find_import_sites_recursive(item, sites);
            }
        }
        Expr::Attrs(bindings) => {
            for b in bindings {
                if let Some(v) = &b.value {
                    find_import_sites_in_value(v, sites);
                }
            }
        }
        Expr::Let { bindings, body } => {
            for b in bindings {
                if let Some(v) = &b.value {
                    find_import_sites_in_let(v, sites);
                }
            }
            find_import_sites_recursive(body, sites);
        }
        Expr::If { cond, then_branch, else_branch } => {
            find_import_sites_recursive(cond, sites);
            find_import_sites_recursive(then_branch, sites);
            find_import_sites_recursive(else_branch, sites);
        }
        Expr::Lambda { body, .. } => {
            find_import_sites_recursive(body, sites);
        }
        Expr::BinaryOp { left, right, .. } => {
            find_import_sites_recursive(left, sites);
            find_import_sites_recursive(right, sites);
        }
        Expr::With { expr, body } | Expr::Assert { cond: expr, body } => {
            find_import_sites_recursive(expr, sites);
            find_import_sites_recursive(body, sites);
        }
        Expr::Negate(inner) => {
            find_import_sites_recursive(inner, sites);
        }
        _ => {}
    }
}

fn find_import_sites_in_value(expr: &Expr, sites: &mut Vec<ImportSite>) {
    if let Expr::Apply { func, arg } = expr {
        if let Expr::Ident(name) = func.as_ref() {
            if name == "import" {
                if let Expr::Path(p) = arg.as_ref() {
                    sites.push(ImportSite {
                        path: p.clone(),
                        span: SourceSpan::new(
                            Position { line: 0, column: 0 },
                            Position { line: 0, column: 0 },
                        ),
                        context: ImportContext::InAttrValue,
                    });
                    return;
                }
            }
        }
    }
    if let Expr::List(items) = expr {
        for item in items {
            if let Expr::Path(p) = item {
                sites.push(ImportSite {
                    path: p.clone(),
                    span: SourceSpan::new(
                        Position { line: 0, column: 0 },
                        Position { line: 0, column: 0 },
                    ),
                    context: ImportContext::InList,
                });
                continue;
            }
            find_import_sites_recursive(item, sites);
        }
        return;
    }
    find_import_sites_recursive(expr, sites);
}

fn find_import_sites_in_let(expr: &Expr, sites: &mut Vec<ImportSite>) {
    if let Expr::Apply { func, arg } = expr {
        if let Expr::Ident(name) = func.as_ref() {
            if name == "import" {
                if let Expr::Path(p) = arg.as_ref() {
                    sites.push(ImportSite {
                        path: p.clone(),
                        span: SourceSpan::new(
                            Position { line: 0, column: 0 },
                            Position { line: 0, column: 0 },
                        ),
                        context: ImportContext::InLetBinding,
                    });
                    return;
                }
            }
        }
    }
    find_import_sites_recursive(expr, sites);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_relative_import() {
        let tmp = std::env::temp_dir().join("avalanche-import-test");
        let _ = std::fs::create_dir_all(&tmp);
        let nix_file = tmp.join("foo.nix");
        std::fs::write(&nix_file, "{}").unwrap();

        let resolved = resolve_import(&tmp, "./foo.nix");
        assert_eq!(resolved, Some(nix_file.clone()));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn resolve_import_with_directory_default() {
        let tmp = std::env::temp_dir().join("avalanche-import-test-dir");
        let subdir = tmp.join("mymod");
        let _ = std::fs::create_dir_all(&subdir);
        std::fs::write(subdir.join("default.nix"), "{}").unwrap();

        let resolved = resolve_import(&tmp, "./mymod");
        assert_eq!(resolved, Some(subdir.join("default.nix")));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn resolve_angle_bracket_returns_none() {
        let resolved = resolve_import(Path::new("/tmp"), "<nixpkgs>");
        assert_eq!(resolved, None);
    }

    #[test]
    fn resolve_nonexistent_returns_none() {
        let resolved = resolve_import(Path::new("/tmp"), "./nonexistent_xyz.nix");
        assert_eq!(resolved, None);
    }

    #[test]
    fn extract_imports_from_parsed_file() {
        let file = parse("{ imports = [ ./a.nix ./b.nix ]; }").unwrap();
        let imports = extract_imports(&file);
        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].path, "./a.nix");
        assert_eq!(imports[1].path, "./b.nix");
    }

    #[test]
    fn find_import_sites_in_list() {
        let file = parse("{ imports = [ ./a.nix ./b.nix ]; }").unwrap();
        let sites = find_import_sites(&file);
        assert_eq!(sites.len(), 2);
        assert_eq!(sites[0].context, ImportContext::InList);
        assert_eq!(sites[1].context, ImportContext::InList);
    }

    #[test]
    fn find_import_sites_top_level() {
        let file = parse("import ./foo.nix").unwrap();
        let sites = find_import_sites(&file);
        assert_eq!(sites.len(), 1);
        assert_eq!(sites[0].context, ImportContext::TopLevel);
    }

    #[test]
    fn normalize_path_removes_dots() {
        let path = PathBuf::from("/a/b/../c/./d.nix");
        let normalized = normalize_path(&path);
        assert_eq!(normalized, PathBuf::from("/a/c/d.nix"));
    }
}
