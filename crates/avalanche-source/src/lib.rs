pub mod ast;
pub mod assignments;
pub mod format;
pub mod imports;
pub mod lexer;
pub mod parser;
pub mod patch;

pub use ast::{BinOp, Binding, Expr, ImportRef, NixFile, SpannedExpr};
pub use assignments::{
    classify_value, extract_assignments, extract_assignments_with_prefix, find_assignment,
    find_assignments_by_prefix, ExtractedAssignment,
};
pub use format::{format_binding, format_expr, format_expr_indented};
pub use imports::{
    build_import_graph, extract_imports, find_import_sites, resolve_import, ImportContext,
    ImportGraph, ImportNode, ImportSite, ResolvedImport,
};
pub use lexer::{tokenize, LexError, Token, TokenKind};
pub use parser::{collect_imports, parse, ParseError, Parser};
pub use patch::{
    apply_patch, apply_patches, generate_binding_insertion_patch,
    generate_binding_removal_patch, generate_value_replacement_patch, PatchError, PatchPlan,
    SourcePatch,
};

use avalanche_model::source::SourceLocation;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentKind {
    DirectLiteral,
    DirectExpression,
    GeneratedPattern,
    Computed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub path: String,
    pub kind: AssignmentKind,
    pub location: SourceLocation,
}

impl Assignment {
    pub fn is_auto_editable(&self) -> bool {
        matches!(
            self.kind,
            AssignmentKind::DirectLiteral | AssignmentKind::GeneratedPattern
        )
    }
}

pub struct SourceService;

impl Default for SourceService {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceService {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_file(&self, content: &str) -> Result<NixFile, ParseError> {
        parse(content)
    }

    pub fn parse_path(&self, path: &Path) -> Result<NixFile, ParseError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            ParseError::Lex(format!("failed to read {}: {}", path.display(), e))
        })?;
        parse(&content)
    }

    pub fn extract_assignments(&self, file: &NixFile) -> Vec<ExtractedAssignment> {
        extract_assignments(file)
    }

    pub fn find_imports(&self, file: &NixFile) -> Vec<ImportRef> {
        extract_imports(file)
    }

    pub fn build_import_graph(
        &self,
        root: &Path,
        max_depth: usize,
    ) -> Result<ImportGraph, ParseError> {
        build_import_graph(root, max_depth)
    }

    pub fn generate_replacement_patch(
        &self,
        file_path: &str,
        source: &str,
        attr_path: &str,
        new_value: &Expr,
    ) -> Result<SourcePatch, PatchError> {
        generate_value_replacement_patch(file_path, source, attr_path, new_value)
    }

    pub fn generate_removal_patch(
        &self,
        file_path: &str,
        source: &str,
        attr_path: &str,
    ) -> Result<SourcePatch, PatchError> {
        generate_binding_removal_patch(file_path, source, attr_path)
    }

    pub fn generate_insertion_patch(
        &self,
        file_path: &str,
        source: &str,
        attr_path: &str,
        value: &Expr,
    ) -> Result<SourcePatch, PatchError> {
        generate_binding_insertion_patch(file_path, source, attr_path, value)
    }

    pub fn apply_patch(&self, source: &str, patch: &SourcePatch) -> Result<String, PatchError> {
        apply_patch(source, patch)
    }

    pub fn apply_patches(
        &self,
        source: &str,
        patches: &[SourcePatch],
    ) -> Result<String, PatchError> {
        apply_patches(source, patches)
    }

    pub fn format(&self, file: &NixFile) -> String {
        format_expr(&file.expr)
    }

    pub fn where_is_value_written(
        &self,
        file: &NixFile,
        file_path: &str,
        option_path: &str,
    ) -> Vec<Assignment> {
        let extracted = extract_assignments(file);
        extracted
            .iter()
            .filter(|a| a.dot_path() == option_path)
            .map(|a| a.to_assignment(file_path))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_parse_and_extract() {
        let svc = SourceService::new();
        let file = svc.parse_file("{ services.pipewire.enable = true; }").unwrap();
        let assignments = svc.extract_assignments(&file);
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].dot_path(), "services.pipewire.enable");
    }

    #[test]
    fn service_where_is_value_written() {
        let svc = SourceService::new();
        let file = svc
            .parse_file("{ services.pipewire.enable = true; gb.schemaVersion = 1; }")
            .unwrap();
        let results = svc.where_is_value_written(&file, "test.nix", "services.pipewire.enable");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].kind, AssignmentKind::DirectLiteral);
        assert_eq!(results[0].location.file, "test.nix");
    }

    #[test]
    fn service_format_roundtrip() {
        let svc = SourceService::new();
        let file = svc.parse_file("{ enable = true; }").unwrap();
        let formatted = svc.format(&file);
        assert!(formatted.contains("enable = true;"));
    }

    #[test]
    fn service_import_detection() {
        let svc = SourceService::new();
        let file = svc.parse_file("{ imports = [ ./a.nix ./b.nix ]; }").unwrap();
        let imports = svc.find_imports(&file);
        assert_eq!(imports.len(), 2);
    }

    #[test]
    fn service_patch_generation() {
        let svc = SourceService::new();
        let source = "{\n  enable = true;\n}\n";
        let patch = svc
            .generate_replacement_patch("test.nix", source, "enable", &Expr::Bool(false))
            .unwrap();
        let result = svc.apply_patch(source, &patch).unwrap();
        assert!(result.contains("false"));
    }

    #[test]
    fn assignment_auto_editable() {
        let literal = Assignment {
            path: "foo".into(),
            kind: AssignmentKind::DirectLiteral,
            location: SourceLocation::new(
                "test.nix",
                avalanche_model::source::SourceSpan::new(
                    avalanche_model::source::Position { line: 1, column: 1 },
                    avalanche_model::source::Position { line: 1, column: 10 },
                ),
            ),
        };
        assert!(literal.is_auto_editable());

        let computed = Assignment {
            path: "bar".into(),
            kind: AssignmentKind::Computed,
            location: literal.location.clone(),
        };
        assert!(!computed.is_auto_editable());

        let unknown = Assignment {
            path: "baz".into(),
            kind: AssignmentKind::Unknown,
            location: literal.location.clone(),
        };
        assert!(!unknown.is_auto_editable());
    }
}
