use avalanche_model::source::SourceSpan;
use serde::{Deserialize, Serialize};

use crate::ast::{Expr, NixFile};
use crate::{Assignment, AssignmentKind};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedAssignment {
    pub path: Vec<String>,
    pub value: Expr,
    pub kind: AssignmentKind,
    pub span: SourceSpan,
}

impl ExtractedAssignment {
    pub fn dot_path(&self) -> String {
        self.path.join(".")
    }

    pub fn to_assignment(&self, file: &str) -> Assignment {
        Assignment {
            path: self.dot_path(),
            kind: self.kind.clone(),
            location: avalanche_model::source::SourceLocation::new(file, self.span),
        }
    }
}

pub fn extract_assignments(file: &NixFile) -> Vec<ExtractedAssignment> {
    let mut assignments = Vec::new();
    walk_bindings(&file.expr, &[], &mut assignments);
    assignments
}

pub fn extract_assignments_with_prefix(
    file: &NixFile,
    prefix: &[String],
) -> Vec<ExtractedAssignment> {
    let mut assignments = Vec::new();
    walk_bindings(&file.expr, prefix, &mut assignments);
    assignments
}

fn walk_bindings(
    expr: &Expr,
    prefix: &[String],
    out: &mut Vec<ExtractedAssignment>,
) {
    match expr {
        Expr::Attrs(bindings) => {
            for binding in bindings {
                if binding.inherit {
                    continue;
                }
                if let Some(value) = &binding.value {
                    let full_path: Vec<String> = prefix
                        .iter()
                        .chain(binding.path.iter())
                        .cloned()
                        .collect();

                    let kind = classify_value(value);
                    out.push(ExtractedAssignment {
                        path: full_path.clone(),
                        value: value.clone(),
                        kind,
                        span: binding.span,
                    });

                    if let Expr::Attrs(_) = value {
                        walk_bindings(value, &full_path, out);
                    }
                }
            }
        }
        Expr::Let { body, .. } => {
            walk_bindings(body, prefix, out);
        }
        Expr::Lambda { body, .. } => {
            walk_bindings(body, prefix, out);
        }
        Expr::With { body, .. } | Expr::Assert { body, .. } => {
            walk_bindings(body, prefix, out);
        }
        _ => {}
    }
}

pub fn classify_value(value: &Expr) -> AssignmentKind {
    match value {
        Expr::Int(_)
        | Expr::Float(_)
        | Expr::Str(_)
        | Expr::Bool(_) => AssignmentKind::DirectLiteral,

        Expr::Path(_) => AssignmentKind::DirectLiteral,

        Expr::List(items) => {
            if items.iter().all(|i| matches!(i, Expr::Str(_) | Expr::Int(_) | Expr::Float(_) | Expr::Bool(_) | Expr::Path(_))) {
                AssignmentKind::DirectLiteral
            } else {
                AssignmentKind::DirectExpression
            }
        }

        Expr::Attrs(bindings) => {
            if bindings.iter().all(|b| {
                b.inherit
                    || b
                        .value
                        .as_ref()
                        .map(|v| matches!(v, Expr::Int(_) | Expr::Float(_) | Expr::Str(_) | Expr::Bool(_) | Expr::Path(_) | Expr::Attrs(_) | Expr::List(_)))
                        .unwrap_or(false)
            }) {
                AssignmentKind::DirectExpression
            } else {
                AssignmentKind::Computed
            }
        }

        Expr::Ident(_) => AssignmentKind::DirectExpression,

        Expr::BinaryOp { .. } => AssignmentKind::DirectExpression,

        Expr::Apply { func, .. } => {
            if let Expr::Ident(name) = func.as_ref() {
                if name == "mkDefault" || name == "mkForce" || name == "mkOverride" {
                    AssignmentKind::DirectExpression
                } else if name == "import" {
                    AssignmentKind::DirectExpression
                } else {
                    AssignmentKind::Computed
                }
            } else {
                AssignmentKind::Computed
            }
        }

        Expr::If { .. } => AssignmentKind::Computed,

        Expr::Lambda { .. } => AssignmentKind::Computed,

        Expr::Let { .. } => AssignmentKind::Computed,

        Expr::With { .. } => AssignmentKind::Computed,

        Expr::Assert { .. } => AssignmentKind::Computed,

        Expr::HasAttr { .. } => AssignmentKind::DirectExpression,

        Expr::Negate(_) => AssignmentKind::DirectExpression,

        Expr::Inherit { .. } => AssignmentKind::Unknown,

        Expr::AttrPath(_) => AssignmentKind::DirectExpression,
    }
}

pub fn find_assignment<'a>(
    assignments: &'a [ExtractedAssignment],
    path: &str,
) -> Option<&'a ExtractedAssignment> {
    assignments.iter().find(|a| a.dot_path() == path)
}

pub fn find_assignments_by_prefix<'a>(
    assignments: &'a [ExtractedAssignment],
    prefix: &str,
) -> Vec<&'a ExtractedAssignment> {
    assignments
        .iter()
        .filter(|a| a.dot_path().starts_with(prefix))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn extract_simple_assignment() {
        let file = parse("{ enable = true; }").unwrap();
        let assignments = extract_assignments(&file);
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].dot_path(), "enable");
        assert_eq!(assignments[0].kind, AssignmentKind::DirectLiteral);
    }

    #[test]
    fn extract_nested_assignment() {
        let file = parse("{ services.pipewire.enable = true; }").unwrap();
        let assignments = extract_assignments(&file);
        let pipewire = find_assignment(&assignments, "services.pipewire.enable");
        assert!(pipewire.is_some());
        assert_eq!(pipewire.unwrap().kind, AssignmentKind::DirectLiteral);
    }

    #[test]
    fn classify_literal_values() {
        assert_eq!(classify_value(&Expr::Bool(true)), AssignmentKind::DirectLiteral);
        assert_eq!(classify_value(&Expr::Int(42)), AssignmentKind::DirectLiteral);
        assert_eq!(classify_value(&Expr::Float(3.14)), AssignmentKind::DirectLiteral);
        assert_eq!(classify_value(&Expr::Str("hello".into())), AssignmentKind::DirectLiteral);
        assert_eq!(classify_value(&Expr::Path("./foo.nix".into())), AssignmentKind::DirectLiteral);
    }

    #[test]
    fn classify_list_of_literals() {
        let list = Expr::List(vec![
            Expr::Str("a".into()),
            Expr::Str("b".into()),
        ]);
        assert_eq!(classify_value(&list), AssignmentKind::DirectLiteral);
    }

    #[test]
    fn classify_computed_expression() {
        let if_expr = Expr::If {
            cond: Box::new(Expr::Bool(true)),
            then_branch: Box::new(Expr::Int(1)),
            else_branch: Box::new(Expr::Int(2)),
        };
        assert_eq!(classify_value(&if_expr), AssignmentKind::Computed);
    }

    #[test]
    fn classify_ident_reference() {
        assert_eq!(
            classify_value(&Expr::Ident("pkgs".into())),
            AssignmentKind::DirectExpression
        );
    }

    #[test]
    fn classify_mk_default() {
        let apply = Expr::Apply {
            func: Box::new(Expr::Ident("mkDefault".into())),
            arg: Box::new(Expr::Bool(true)),
        };
        assert_eq!(classify_value(&apply), AssignmentKind::DirectExpression);
    }

    #[test]
    fn classify_unknown_function() {
        let apply = Expr::Apply {
            func: Box::new(Expr::Ident("someComplexFunction".into())),
            arg: Box::new(Expr::Bool(true)),
        };
        assert_eq!(classify_value(&apply), AssignmentKind::Computed);
    }

    #[test]
    fn find_by_prefix() {
        let file = parse("{ services.a.enable = true; services.b.enable = false; networking.enable = true; }").unwrap();
        let assignments = extract_assignments(&file);
        let services = find_assignments_by_prefix(&assignments, "services");
        assert_eq!(services.len(), 2);
    }

    #[test]
    fn extract_from_fixture_pattern() {
        let input = r#"{
  services.pipewire.enable = true;
  gb.schemaVersion = 1;
  gb.programs.git.enable = true;
}"#;
        let file = parse(input).unwrap();
        let assignments = extract_assignments(&file);
        assert!(find_assignment(&assignments, "services.pipewire.enable").is_some());
        assert!(find_assignment(&assignments, "gb.schemaVersion").is_some());
        assert!(find_assignment(&assignments, "gb.programs.git.enable").is_some());
    }
}
