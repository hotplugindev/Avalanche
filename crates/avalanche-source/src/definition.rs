use avalanche_model::{Condition, DefinitionType, JsonValue, Priority};
use serde::{Deserialize, Serialize};

use crate::ast::Expr;
use crate::format::format_expr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefinitionInfo {
    pub value: Option<JsonValue>,
    pub definition_type: DefinitionType,
    pub priority: Priority,
    pub condition: Option<Condition>,
}

pub fn expr_to_json(expr: &Expr) -> Option<JsonValue> {
    match expr {
        Expr::Bool(v) => Some(JsonValue(serde_json::Value::Bool(*v))),
        Expr::Int(v) => Some(JsonValue(serde_json::Value::Number((*v).into()))),
        Expr::Float(v) => {
            serde_json::Number::from_f64(*v).map(|n| JsonValue(serde_json::Value::Number(n)))
        }
        Expr::Str(v) => Some(JsonValue(serde_json::Value::String(v.clone()))),
        Expr::Path(v) => Some(JsonValue(serde_json::Value::String(v.clone()))),
        Expr::List(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(expr_to_json(item)?.0);
            }
            Some(JsonValue(serde_json::Value::Array(out)))
        }
        Expr::Attrs(bindings) => {
            let mut out = serde_json::Map::new();
            for binding in bindings {
                if binding.inherit {
                    return None;
                }
                let key = binding.path.join(".");
                let value = binding.value.as_ref().and_then(expr_to_json)?;
                out.insert(key, value.0);
            }
            Some(JsonValue(serde_json::Value::Object(out)))
        }
        _ => None,
    }
}

pub fn analyze_definition(expr: &Expr) -> DefinitionInfo {
    match expr {
        Expr::Apply { func, arg } => {
            if let Some(name) = helper_name(func) {
                match name {
                    "mkDefault" => {
                        let inner = analyze_definition(arg);
                        return DefinitionInfo {
                            value: inner.value,
                            definition_type: DefinitionType::MkDefault,
                            priority: Priority::mk_default(),
                            condition: inner.condition,
                        };
                    }
                    "mkForce" => {
                        let inner = analyze_definition(arg);
                        return DefinitionInfo {
                            value: inner.value,
                            definition_type: DefinitionType::MkForce,
                            priority: Priority::mk_force(),
                            condition: inner.condition,
                        };
                    }
                    "mkMerge" => {
                        let inner = analyze_definition(arg);
                        return DefinitionInfo {
                            value: inner.value,
                            definition_type: DefinitionType::MkMerge,
                            priority: inner.priority,
                            condition: inner.condition,
                        };
                    }
                    _ => {}
                }
            }

            if let Expr::Apply {
                func: inner_func,
                arg: inner_arg,
            } = func.as_ref()
            {
                if let Some(name) = helper_name(inner_func) {
                    match name {
                        "mkOverride" => {
                            if let Expr::Int(p) = inner_arg.as_ref() {
                                let inner = analyze_definition(arg);
                                let priority = (*p).clamp(i16::MIN as i64, i16::MAX as i64) as i16;
                                return DefinitionInfo {
                                    value: inner.value,
                                    definition_type: DefinitionType::MkOverride,
                                    priority: Priority(priority),
                                    condition: inner.condition,
                                };
                            }
                        }
                        "mkIf" => {
                            let inner = analyze_definition(arg);
                            return DefinitionInfo {
                                value: inner.value,
                                definition_type: DefinitionType::MkIf,
                                priority: inner.priority,
                                condition: Some(Condition {
                                    expression: format_expr(inner_arg),
                                    evaluated_value: None,
                                    satisfied: None,
                                }),
                            };
                        }
                        _ => {}
                    }
                }
            }

            DefinitionInfo {
                value: expr_to_json(arg),
                definition_type: DefinitionType::Unknown,
                priority: Priority::normal(),
                condition: None,
            }
        }
        Expr::Bool(_)
        | Expr::Int(_)
        | Expr::Float(_)
        | Expr::Str(_)
        | Expr::Path(_)
        | Expr::List(_)
        | Expr::Attrs(_) => DefinitionInfo {
            value: expr_to_json(expr),
            definition_type: DefinitionType::Literal,
            priority: Priority::normal(),
            condition: None,
        },
        Expr::Ident(_)
        | Expr::AttrPath(_)
        | Expr::BinaryOp { .. }
        | Expr::HasAttr { .. }
        | Expr::Negate(_) => DefinitionInfo {
            value: expr_to_json(expr),
            definition_type: DefinitionType::Expression,
            priority: Priority::normal(),
            condition: None,
        },
        Expr::If { .. }
        | Expr::Lambda { .. }
        | Expr::Let { .. }
        | Expr::With { .. }
        | Expr::Assert { .. }
        | Expr::Inherit { .. } => DefinitionInfo {
            value: None,
            definition_type: DefinitionType::Unknown,
            priority: Priority::normal(),
            condition: None,
        },
    }
}

fn helper_name(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Ident(name) => Some(name.as_str()),
        Expr::AttrPath(parts) => parts.last().map(String::as_str),
        _ => None,
    }
}
