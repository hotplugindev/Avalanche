use crate::ast::{Binding, Expr};

pub fn format_expr(expr: &Expr) -> String {
    format_expr_inner(expr, 0)
}

pub fn format_expr_indented(expr: &Expr, indent: usize) -> String {
    format_expr_inner(expr, indent)
}

fn indent_str(level: usize) -> String {
    "  ".repeat(level)
}

fn format_expr_inner(expr: &Expr, indent: usize) -> String {
    match expr {
        Expr::Int(n) => n.to_string(),
        Expr::Float(f) => format_float(*f),
        Expr::Str(s) => format!("\"{}\"", escape_string(s)),
        Expr::Bool(b) => b.to_string(),
        Expr::Path(p) => p.clone(),
        Expr::Ident(name) => name.clone(),
        Expr::AttrPath(path) => path.join("."),
        Expr::Attrs(bindings) => format_attrs(bindings, indent),
        Expr::List(items) => format_list(items, indent),
        Expr::Let { bindings, body } => format_let(bindings, body, indent),
        Expr::If {
            cond,
            then_branch,
            else_branch,
        } => format!(
            "if {} then {} else {}",
            format_expr_inner(cond, indent),
            format_expr_inner(then_branch, indent),
            format_expr_inner(else_branch, indent)
        ),
        Expr::Apply { func, arg } => format!(
            "{} {}",
            format_apply_target(func, indent),
            format_apply_target(arg, indent)
        ),
        Expr::Lambda { param, body } => format!(
            "{}: {}",
            param,
            format_expr_inner(body, indent)
        ),
        Expr::BinaryOp { op, left, right } => format!(
            "{} {} {}",
            format_apply_target(left, indent),
            op.as_str(),
            format_apply_target(right, indent)
        ),
        Expr::HasAttr { expr: e, attr } => format!(
            "{} ? {}",
            format_apply_target(e, indent),
            attr.join(".")
        ),
        Expr::Negate(inner) => format!("!{}", format_expr_inner(inner, indent)),
        Expr::With { expr: e, body } => format!(
            "with {}; {}",
            format_expr_inner(e, indent),
            format_expr_inner(body, indent)
        ),
        Expr::Assert { cond, body } => format!(
            "assert {}; {}",
            format_expr_inner(cond, indent),
            format_expr_inner(body, indent)
        ),
        Expr::Inherit { names, from } => {
            if let Some(from_expr) = from {
                format!("inherit ({}) {};", format_expr_inner(from_expr, indent), names.join(" "))
            } else {
                format!("inherit {};", names.join(" "))
            }
        }
    }
}

fn format_apply_target(expr: &Expr, indent: usize) -> String {
    match expr {
        Expr::BinaryOp { .. } | Expr::If { .. } | Expr::Let { .. } => {
            format!("({})", format_expr_inner(expr, indent))
        }
        _ => format_expr_inner(expr, indent),
    }
}

fn format_attrs(bindings: &[Binding], indent: usize) -> String {
    if bindings.is_empty() {
        return "{}".to_string();
    }

    let inner_indent = indent + 1;
    let mut lines = Vec::new();
    lines.push("{".to_string());

    for binding in bindings {
        if binding.inherit {
            if let Some(Expr::Inherit { names, from }) = &binding.value {
                if let Some(from_expr) = from {
                    lines.push(format!(
                        "{}inherit ({}) {};",
                        indent_str(inner_indent),
                        format_expr_inner(from_expr, inner_indent),
                        names.join(" ")
                    ));
                } else {
                    lines.push(format!(
                        "{}inherit {};",
                        indent_str(inner_indent),
                        names.join(" ")
                    ));
                }
            }
        } else if let Some(value) = &binding.value {
            let path = binding.path.join(".");
            let formatted_value = format_expr_inner(value, inner_indent);
            lines.push(format!(
                "{}{} = {};",
                indent_str(inner_indent),
                path,
                formatted_value
            ));
        }
    }

    lines.push(format!("{}}}", indent_str(indent)));
    lines.join("\n")
}

fn format_list(items: &[Expr], indent: usize) -> String {
    if items.is_empty() {
        return "[]".to_string();
    }

    let all_simple = items.iter().all(|i| {
        matches!(
            i,
            Expr::Int(_) | Expr::Float(_) | Expr::Str(_) | Expr::Bool(_) | Expr::Path(_) | Expr::Ident(_)
        )
    });

    if all_simple {
        let parts: Vec<String> = items
            .iter()
            .map(|i| format_expr_inner(i, indent))
            .collect();
        format!("[ {} ]", parts.join(" "))
    } else {
        let inner_indent = indent + 1;
        let mut lines = Vec::new();
        lines.push("[".to_string());
        for item in items {
            lines.push(format!(
                "{}{}",
                indent_str(inner_indent),
                format_expr_inner(item, inner_indent)
            ));
        }
        lines.push(format!("{}]", indent_str(indent)));
        lines.join("\n")
    }
}

fn format_let(bindings: &[Binding], body: &Expr, indent: usize) -> String {
    let inner_indent = indent + 1;
    let mut lines = Vec::new();
    lines.push("let".to_string());

    for binding in bindings {
        if let Some(value) = &binding.value {
            let path = binding.path.join(".");
            lines.push(format!(
                "{}{} = {};",
                indent_str(inner_indent),
                path,
                format_expr_inner(value, inner_indent)
            ));
        }
    }

    lines.push(format!("{}in", indent_str(indent)));
    lines.push(format!(
        "{}{}",
        indent_str(indent),
        format_expr_inner(body, indent)
    ));
    lines.join("\n")
}

fn format_float(f: f64) -> String {
    if f.fract() == 0.0 && f.abs() < 1e15 {
        format!("{:.1}", f)
    } else {
        format!("{}", f)
    }
}

fn escape_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\t' => "\\t".to_string(),
            '\r' => "\\r".to_string(),
            '$' => "\\$".to_string(),
            other => other.to_string(),
        })
        .collect()
}

pub fn format_binding(binding: &Binding, indent: usize) -> String {
    if binding.inherit {
        if let Some(Expr::Inherit { names, from }) = &binding.value {
            if let Some(from_expr) = from {
                return format!(
                    "inherit ({}) {};",
                    format_expr_inner(from_expr, indent),
                    names.join(" ")
                );
            }
            return format!("inherit {};", names.join(" "));
        }
        return "inherit;".to_string();
    }

    if let Some(value) = &binding.value {
        format!(
            "{} = {};",
            binding.path.join("."),
            format_expr_inner(value, indent)
        )
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::BinOp;
    use crate::parser::parse;

    #[test]
    fn format_bool() {
        assert_eq!(format_expr(&Expr::Bool(true)), "true");
        assert_eq!(format_expr(&Expr::Bool(false)), "false");
    }

    #[test]
    fn format_int() {
        assert_eq!(format_expr(&Expr::Int(42)), "42");
        assert_eq!(format_expr(&Expr::Int(-1)), "-1");
    }

    #[test]
    fn format_string() {
        assert_eq!(format_expr(&Expr::Str("hello".into())), "\"hello\"");
        assert_eq!(
            format_expr(&Expr::Str("say \"hi\"".into())),
            "\"say \\\"hi\\\"\""
        );
    }

    #[test]
    fn format_simple_attrs() {
        let file = parse("{ enable = true; count = 42; }").unwrap();
        let formatted = format_expr(&file.expr);
        assert!(formatted.contains("enable = true;"));
        assert!(formatted.contains("count = 42;"));
    }

    #[test]
    fn format_empty_attrs() {
        assert_eq!(format_expr(&Expr::Attrs(vec![])), "{}");
    }

    #[test]
    fn format_list_of_strings() {
        let list = Expr::List(vec![
            Expr::Str("a".into()),
            Expr::Str("b".into()),
        ]);
        assert_eq!(format_expr(&list), "[ \"a\" \"b\" ]");
    }

    #[test]
    fn format_binary_op() {
        let expr = Expr::BinaryOp {
            op: BinOp::Add,
            left: Box::new(Expr::Int(1)),
            right: Box::new(Expr::Int(2)),
        };
        assert_eq!(format_expr(&expr), "1 + 2");
    }

    #[test]
    fn format_lambda() {
        let expr = Expr::Lambda {
            param: "x".into(),
            body: Box::new(Expr::Ident("x".into())),
        };
        assert_eq!(format_expr(&expr), "x: x");
    }

    #[test]
    fn format_if() {
        let expr = Expr::If {
            cond: Box::new(Expr::Bool(true)),
            then_branch: Box::new(Expr::Int(1)),
            else_branch: Box::new(Expr::Int(2)),
        };
        assert_eq!(format_expr(&expr), "if true then 1 else 2");
    }

    #[test]
    fn format_nested_attrs() {
        let file = parse("{ a = { b = 1; }; }").unwrap();
        let formatted = format_expr(&file.expr);
        assert!(formatted.contains("a = {"));
        assert!(formatted.contains("b = 1;"));
    }

    #[test]
    fn format_let_in() {
        let file = parse("let x = 1; in x").unwrap();
        let formatted = format_expr(&file.expr);
        assert!(formatted.contains("let"));
        assert!(formatted.contains("x = 1;"));
        assert!(formatted.contains("in"));
    }

    #[test]
    fn format_has_attr() {
        let expr = Expr::HasAttr {
            expr: Box::new(Expr::Ident("attrs".into())),
            attr: vec!["foo".into()],
        };
        assert_eq!(format_expr(&expr), "attrs ? foo");
    }

    #[test]
    fn format_path() {
        assert_eq!(format_expr(&Expr::Path("./foo.nix".into())), "./foo.nix");
    }

    #[test]
    fn format_float_value() {
        assert_eq!(format_expr(&Expr::Float(3.14)), "3.14");
        assert_eq!(format_expr(&Expr::Float(1.0)), "1.0");
    }
}
