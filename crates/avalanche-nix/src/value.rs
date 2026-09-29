use avalanche_model::{JsonValue, NixType, NixValue};
use serde_json::Value;
use std::collections::BTreeMap;

pub fn decode_json(raw: &str) -> Result<Value, crate::error::NixError> {
    serde_json::from_str(raw).map_err(|e| crate::error::NixError::JsonDecodeFailed {
        reason: e.to_string(),
    })
}

pub fn json_to_nix_value(value: &Value) -> NixValue {
    match value {
        Value::Null => NixValue::Null,
        Value::Bool(b) => NixValue::Bool(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                NixValue::Int(i)
            } else {
                NixValue::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => NixValue::Str(s.clone()),
        Value::Array(arr) => {
            NixValue::List(arr.iter().map(json_to_nix_value).collect())
        }
        Value::Object(map) => {
            let mut attrs = BTreeMap::new();
            for (k, v) in map {
                attrs.insert(k.clone(), json_to_nix_value(v));
            }
            NixValue::Attrs(attrs)
        }
    }
}

pub fn infer_nix_type(value: &Value) -> NixType {
    match value {
        Value::Null => NixType::Unknown,
        Value::Bool(_) => NixType::Bool,
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                NixType::Int
            } else {
                NixType::Float
            }
        }
        Value::String(_) => NixType::Str,
        Value::Array(_) => NixType::List,
        Value::Object(map) => {
            if map.contains_key("_type") {
                let type_str = map["_type"].as_str().unwrap_or("");
                match type_str {
                    "derivation" => NixType::Package,
                    "enum" => NixType::Enum,
                    "lambda" => NixType::Function,
                    _ => NixType::Raw,
                }
            } else {
                NixType::Attrs
            }
        }
    }
}

pub fn value_to_json(value: &NixValue) -> JsonValue {
    JsonValue(nix_value_to_serde(value))
}

fn nix_value_to_serde(value: &NixValue) -> Value {
    match value {
        NixValue::Null => Value::Null,
        NixValue::Bool(b) => Value::Bool(*b),
        NixValue::Int(i) => Value::Number((*i).into()),
        NixValue::Float(f) => {
            serde_json::Number::from_f64(*f)
                .map(Value::Number)
                .unwrap_or(Value::Null)
        }
        NixValue::Str(s) => Value::String(s.clone()),
        NixValue::Path(p) => Value::String(p.clone()),
        NixValue::List(items) => {
            Value::Array(items.iter().map(nix_value_to_serde).collect())
        }
        NixValue::Attrs(map) => {
            let obj: serde_json::Map<String, Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), nix_value_to_serde(v)))
                .collect();
            Value::Object(obj)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_json_bool() {
        let v = decode_json("true").unwrap();
        assert_eq!(infer_nix_type(&v), NixType::Bool);
    }

    #[test]
    fn decode_json_string() {
        let v = decode_json(r#""hello""#).unwrap();
        assert_eq!(infer_nix_type(&v), NixType::Str);
    }

    #[test]
    fn decode_json_int() {
        let v = decode_json("42").unwrap();
        assert_eq!(infer_nix_type(&v), NixType::Int);
    }

    #[test]
    fn decode_json_float() {
        let v = decode_json("3.14").unwrap();
        assert_eq!(infer_nix_type(&v), NixType::Float);
    }

    #[test]
    fn decode_json_list() {
        let v = decode_json("[1, 2, 3]").unwrap();
        assert_eq!(infer_nix_type(&v), NixType::List);
    }

    #[test]
    fn decode_json_attrs() {
        let v = decode_json(r#"{"enable": true, "name": "test"}"#).unwrap();
        assert_eq!(infer_nix_type(&v), NixType::Attrs);
    }

    #[test]
    fn decode_json_package() {
        let v = decode_json(r#"{"_type": "derivation", "name": "foo"}"#).unwrap();
        assert_eq!(infer_nix_type(&v), NixType::Package);
    }

    #[test]
    fn json_to_nix_value_roundtrip() {
        let v = decode_json(r#"{"a": true, "b": [1, "x"]}"#).unwrap();
        let nv = json_to_nix_value(&v);
        match &nv {
            NixValue::Attrs(map) => {
                assert_eq!(map.get("a"), Some(&NixValue::Bool(true)));
                assert!(map.contains_key("b"));
            }
            _ => panic!("expected attrs"),
        }
    }

    #[test]
    fn decode_invalid_json_fails() {
        assert!(decode_json("not json{{{").is_err());
    }
}
