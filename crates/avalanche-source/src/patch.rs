use avalanche_model::source::SourceSpan;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ast::{Binding, Expr, NixFile};
use crate::format::format_expr;
use crate::parser::parse;

#[derive(Debug, Error)]
pub enum PatchError {
    #[error("span out of bounds: line {line}, column {col}")]
    SpanOutOfBounds { line: u32, col: u32 },
    #[error("assignment not found: {path}")]
    AssignmentNotFound { path: String },
    #[error("parse error: {0}")]
    Parse(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourcePatch {
    pub file: String,
    pub span: SourceSpan,
    pub old_text: String,
    pub new_text: String,
    pub description: String,
}

impl SourcePatch {
    pub fn new(
        file: impl Into<String>,
        span: SourceSpan,
        old_text: impl Into<String>,
        new_text: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            file: file.into(),
            span,
            old_text: old_text.into(),
            new_text: new_text.into(),
            description: description.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PatchPlan {
    pub file: String,
    pub patches: Vec<SourcePatch>,
}

impl PatchPlan {
    pub fn new(file: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            patches: Vec::new(),
        }
    }

    pub fn add(&mut self, patch: SourcePatch) {
        self.patches.push(patch);
    }

    pub fn is_empty(&self) -> bool {
        self.patches.is_empty()
    }
}

pub fn apply_patch(source: &str, patch: &SourcePatch) -> Result<String, PatchError> {
    let lines: Vec<&str> = source.lines().collect();
    let start_line = (patch.span.start.line as usize).saturating_sub(1);
    let end_line = (patch.span.end.line as usize).saturating_sub(1);

    if start_line >= lines.len() || end_line >= lines.len() {
        return Err(PatchError::SpanOutOfBounds {
            line: patch.span.start.line,
            col: patch.span.start.column,
        });
    }

    let start_col = (patch.span.start.column as usize).saturating_sub(1);
    let end_col = (patch.span.end.column as usize).saturating_sub(1);

    let mut result = String::new();

    for (i, line) in lines.iter().enumerate() {
        if i < start_line {
            result.push_str(line);
            result.push('\n');
        } else if i == start_line && start_line == end_line {
            let prefix = &line[..start_col.min(line.len())];
            let suffix = &line[end_col.min(line.len())..];
            result.push_str(prefix);
            result.push_str(&patch.new_text);
            result.push_str(suffix);
            result.push('\n');
        } else if i == start_line {
            let prefix = &line[..start_col.min(line.len())];
            result.push_str(prefix);
            result.push_str(&patch.new_text);
            result.push('\n');
        } else if i == end_line {
            let suffix = &line[end_col.min(line.len())..];
            result.push_str(suffix);
            result.push('\n');
        } else if i > start_line && i < end_line {
            // skip lines within multi-line span
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    if result.ends_with('\n') && !source.ends_with('\n') {
        result.pop();
    }

    Ok(result)
}

pub fn apply_patches(source: &str, patches: &[SourcePatch]) -> Result<String, PatchError> {
    let mut current = source.to_string();
    let mut sorted: Vec<&SourcePatch> = patches.iter().collect();
    sorted.sort_by(|a, b| {
        b.span
            .start
            .line
            .cmp(&a.span.start.line)
            .then(b.span.start.column.cmp(&a.span.start.column))
    });

    for patch in sorted {
        current = apply_patch(&current, patch)?;
    }

    Ok(current)
}

pub fn generate_value_replacement_patch(
    file_path: &str,
    source: &str,
    attr_path: &str,
    new_value: &Expr,
) -> Result<SourcePatch, PatchError> {
    let nix_file = parse(source).map_err(|e| PatchError::Parse(e.to_string()))?;

    let binding = find_binding_by_path(&nix_file, attr_path)?;
    let new_text = format_expr(new_value);
    let old_text = binding
        .value
        .as_ref()
        .map(|v| format_expr(v))
        .unwrap_or_default();

    Ok(SourcePatch::new(
        file_path,
        binding.span,
        old_text,
        new_text,
        format!("Replace value of {attr_path}"),
    ))
}

pub fn generate_binding_removal_patch(
    file_path: &str,
    source: &str,
    attr_path: &str,
) -> Result<SourcePatch, PatchError> {
    let nix_file = parse(source).map_err(|e| PatchError::Parse(e.to_string()))?;

    let binding = find_binding_by_path(&nix_file, attr_path)?;

    Ok(SourcePatch::new(
        file_path,
        binding.span,
        format_binding_text(&binding),
        String::new(),
        format!("Remove binding {attr_path}"),
    ))
}

pub fn generate_binding_insertion_patch(
    file_path: &str,
    source: &str,
    attr_path: &str,
    value: &Expr,
) -> Result<SourcePatch, PatchError> {
    let nix_file = parse(source).map_err(|e| PatchError::Parse(e.to_string()))?;

    let parent_path: Vec<&str> = attr_path.rsplitn(2, '.').nth(1).map(|p| p.split('.').collect()).unwrap_or_default();

    let parent_binding = if parent_path.is_empty() {
        None
    } else {
        find_binding_by_path_parts(&nix_file, &parent_path).ok()
    };

    let indent = if let Some(parent) = &parent_binding {
        let line_num = parent.span.start.line as usize;
        let lines: Vec<&str> = source.lines().collect();
        if line_num > 0 && line_num <= lines.len() {
            let line = lines[line_num - 1];
            let leading_spaces = line.len() - line.trim_start().len();
            " ".repeat(leading_spaces + 2)
        } else {
            "  ".to_string()
        }
    } else {
        "  ".to_string()
    };

    let new_binding_text = format!("{}{} = {};", indent, attr_path, format_expr(value));

    let insert_span = if let Some(parent) = &parent_binding {
        avalanche_model::source::SourceSpan::new(
            parent.span.end,
            parent.span.end,
        )
    } else {
        let end_line = source.lines().count() as u32;
        avalanche_model::source::SourceSpan::new(
            avalanche_model::source::Position {
                line: end_line,
                column: 1,
            },
            avalanche_model::source::Position {
                line: end_line,
                column: 1,
            },
        )
    };

    Ok(SourcePatch::new(
        file_path,
        insert_span,
        String::new(),
        format!("\n{}", new_binding_text),
        format!("Insert binding {attr_path}"),
    ))
}

fn find_binding_by_path<'a>(
    file: &'a NixFile,
    attr_path: &str,
) -> Result<Binding, PatchError> {
    let parts: Vec<&str> = attr_path.split('.').collect();
    find_binding_by_path_parts(file, &parts)
}

fn find_binding_by_path_parts(
    file: &NixFile,
    parts: &[&str],
) -> Result<Binding, PatchError> {
    find_binding_in_expr(&file.expr, parts)
}

fn find_binding_in_expr(expr: &Expr, parts: &[&str]) -> Result<Binding, PatchError> {
    match expr {
        Expr::Attrs(bindings) => {
            for binding in bindings {
                if binding.path.first().map(|s| s.as_str()) == parts.first().copied() {
                    if binding.path.len() == parts.len()
                        && binding.path.iter().zip(parts.iter()).all(|(a, b)| a == b)
                    {
                        return Ok(binding.clone());
                    }
                    if parts.len() > binding.path.len() {
                        let remaining = &parts[binding.path.len()..];
                        if let Some(value) = &binding.value {
                            return find_binding_in_expr(value, remaining);
                        }
                    }
                }
                if let Some(value) = &binding.value {
                    if let Ok(found) = find_binding_in_expr(value, parts) {
                        return Ok(found);
                    }
                }
            }
            Err(PatchError::AssignmentNotFound {
                path: parts.join("."),
            })
        }
        Expr::Let { body, .. } | Expr::Lambda { body, .. } => {
            find_binding_in_expr(body, parts)
        }
        Expr::With { body, .. } | Expr::Assert { body, .. } => {
            find_binding_in_expr(body, parts)
        }
        _ => Err(PatchError::AssignmentNotFound {
            path: parts.join("."),
        }),
    }
}

fn format_binding_text(binding: &Binding) -> String {
    if let Some(value) = &binding.value {
        format!("{} = {};", binding.path.join("."), format_expr(value))
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_simple_patch() {
        let source = "{\n  enable = true;\n}\n";
        let patch = SourcePatch::new(
            "test.nix",
            avalanche_model::source::SourceSpan::new(
                avalanche_model::source::Position { line: 2, column: 3 },
                avalanche_model::source::Position { line: 2, column: 18 },
            ),
            "enable = true;",
            "enable = false;",
            "test patch",
        );
        let result = apply_patch(source, &patch).unwrap();
        assert!(result.contains("enable = false;"));
    }

    #[test]
    fn apply_patches_reverse_order() {
        let source = "line1\nline2\nline3\n";
        let patch1 = SourcePatch::new(
            "test.nix",
            avalanche_model::source::SourceSpan::new(
                avalanche_model::source::Position { line: 1, column: 1 },
                avalanche_model::source::Position { line: 1, column: 6 },
            ),
            "line1",
            "LINE1",
            "patch1",
        );
        let patch2 = SourcePatch::new(
            "test.nix",
            avalanche_model::source::SourceSpan::new(
                avalanche_model::source::Position { line: 3, column: 1 },
                avalanche_model::source::Position { line: 3, column: 6 },
            ),
            "line3",
            "LINE3",
            "patch2",
        );
        let result = apply_patches(source, &[patch1, patch2]).unwrap();
        assert!(result.contains("LINE1"));
        assert!(result.contains("LINE3"));
    }

    #[test]
    fn generate_value_replacement() {
        let source = "{\n  enable = true;\n}\n";
        let patch = generate_value_replacement_patch(
            "test.nix",
            source,
            "enable",
            &Expr::Bool(false),
        )
        .unwrap();
        assert_eq!(patch.new_text, "false");
        assert!(patch.description.contains("enable"));
    }

    #[test]
    fn generate_value_replacement_nested() {
        let source = "{\n  services.pipewire.enable = true;\n}\n";
        let patch = generate_value_replacement_patch(
            "test.nix",
            source,
            "services.pipewire.enable",
            &Expr::Bool(false),
        )
        .unwrap();
        assert_eq!(patch.new_text, "false");
    }

    #[test]
    fn generate_binding_removal() {
        let source = "{\n  enable = true;\n}\n";
        let patch = generate_binding_removal_patch("test.nix", source, "enable").unwrap();
        assert_eq!(patch.new_text, "");
    }

    #[test]
    fn patch_plan() {
        let mut plan = PatchPlan::new("test.nix");
        assert!(plan.is_empty());

        plan.add(SourcePatch::new(
            "test.nix",
            avalanche_model::source::SourceSpan::new(
                avalanche_model::source::Position { line: 1, column: 1 },
                avalanche_model::source::Position { line: 1, column: 2 },
            ),
            "a",
            "b",
            "test",
        ));
        assert!(!plan.is_empty());
        assert_eq!(plan.patches.len(), 1);
    }

    #[test]
    fn assignment_not_found() {
        let source = "{ enable = true; }";
        let result = generate_value_replacement_patch(
            "test.nix",
            source,
            "nonexistent",
            &Expr::Bool(false),
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            PatchError::AssignmentNotFound { path } => {
                assert_eq!(path, "nonexistent");
            }
            other => panic!("expected AssignmentNotFound, got {:?}", other),
        }
    }

    #[test]
    fn generate_insertion_patch() {
        let source = "{\n  existing = true;\n}\n";
        let patch = generate_binding_insertion_patch(
            "test.nix",
            source,
            "new_option",
            &Expr::Bool(true),
        )
        .unwrap();
        assert!(patch.new_text.contains("new_option = true;"));
    }
}
