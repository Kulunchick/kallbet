//! Checks a provider's answer against the request's JSON Schema in Rust, so a
//! provider that ignores or loosens the schema still cannot hand back a wrong shape.
//! Supports the subset the role schemas use: `type` (one or a list), `properties`,
//! `required`, `additionalProperties: false`, `items`, `enum`, `const`,
//! `minimum`/`maximum`, `minItems`/`maxItems`, `minLength`/`maxLength`. Other
//! keywords are ignored, so keep role schemas inside this subset.
use serde_json::Value;

pub fn valid(value: &Value, schema: &Value) -> bool {
    let Some(schema) = schema.as_object() else {
        return true;
    };
    if let Some(t) = schema.get("type") {
        let types: Vec<&str> = match t {
            Value::String(s) => vec![s.as_str()],
            Value::Array(a) => a.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        };
        if !types.iter().any(|t| is_type(value, t)) {
            return false;
        }
    }
    if let Some(options) = schema.get("enum").and_then(Value::as_array)
        && !options.contains(value)
    {
        return false;
    }
    if schema.get("const").is_some_and(|c| c != value) {
        return false;
    }
    let num = |k: &str| schema.get(k).and_then(Value::as_f64);
    let count = |k: &str| schema.get(k).and_then(Value::as_u64).map(|n| n as usize);
    match value {
        Value::Number(n) => {
            let n = n.as_f64().unwrap_or(f64::NAN);
            if num("minimum").is_some_and(|m| n < m) || num("maximum").is_some_and(|m| n > m) {
                return false;
            }
        }
        Value::String(s) => {
            let len = s.chars().count();
            if count("minLength").is_some_and(|m| len < m)
                || count("maxLength").is_some_and(|m| len > m)
            {
                return false;
            }
        }
        Value::Array(items) => {
            if count("minItems").is_some_and(|m| items.len() < m)
                || count("maxItems").is_some_and(|m| items.len() > m)
            {
                return false;
            }
            if let Some(item) = schema.get("items")
                && !items.iter().all(|v| valid(v, item))
            {
                return false;
            }
        }
        Value::Object(fields) => {
            let properties = schema.get("properties").and_then(Value::as_object);
            if let Some(required) = schema.get("required").and_then(Value::as_array)
                && !required
                    .iter()
                    .filter_map(Value::as_str)
                    .all(|k| fields.contains_key(k))
            {
                return false;
            }
            for (k, v) in fields {
                match properties.and_then(|p| p.get(k)) {
                    Some(s) if !valid(v, s) => return false,
                    Some(_) => {}
                    None if schema.get("additionalProperties") == Some(&Value::Bool(false)) => {
                        return false;
                    }
                    None => {}
                }
            }
        }
        _ => {}
    }
    true
}

fn is_type(value: &Value, t: &str) -> bool {
    match t {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.as_f64().is_some_and(|n| n.fract() == 0.0),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adjustment_schema;
    use serde_json::json;
    #[test]
    fn adjustment_shape() {
        let ok = json!({"delta": 3.5, "factors": [{"kind": "stand_in", "evidence_url": "https://www.hltv.org/news/1"}],
            "summary": {"en": "a", "ru": "b", "uk": "c"}});
        assert!(valid(&ok, &adjustment_schema()));
        let mut bad = ok.clone();
        bad["factors"][0]["kind"] = json!("rumour");
        assert!(!valid(&bad, &adjustment_schema()));
        let mut extra = ok.clone();
        extra["confidence"] = json!(1);
        assert!(!valid(&extra, &adjustment_schema()));
        let mut missing = ok;
        missing["summary"].as_object_mut().unwrap().remove("uk");
        assert!(!valid(&missing, &adjustment_schema()));
    }
    #[test]
    fn integers_and_bounds() {
        let s = json!({"type": "integer", "minimum": 0, "maximum": 10});
        assert!(valid(&json!(5), &s));
        assert!(!valid(&json!(5.5), &s));
        assert!(!valid(&json!(11), &s));
        assert!(!valid(&json!("5"), &s));
    }
}
