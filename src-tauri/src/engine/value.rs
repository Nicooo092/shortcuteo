use std::fmt;
use std::path::PathBuf;

use chrono::{DateTime, FixedOffset};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A value flowing between actions.
///
/// Every action output is a `Value`, and any later action can take it as input. Conversions
/// between kinds are implicit and lenient, see [`Value::coerce`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
#[ts(export)]
pub enum Value {
    Nothing,
    Text(String),
    Number(f64),
    Bool(bool),
    Date(DateTime<FixedOffset>),
    List(Vec<Value>),
    Dict(IndexMap<String, Value>),
    File(PathBuf),
    Image(PathBuf),
    Url(String),
}

/// Declared type of an action input or output. `Any` accepts everything as is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum ValueType {
    Nothing,
    Text,
    Number,
    Bool,
    Date,
    List,
    Dict,
    File,
    Image,
    Url,
    Any,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("cannot use {from} as {to}")]
pub struct ConversionError {
    pub from: ValueType,
    pub to: ValueType,
}

impl fmt::Display for ValueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ValueType::Nothing => "nothing",
            ValueType::Text => "text",
            ValueType::Number => "number",
            ValueType::Bool => "boolean",
            ValueType::Date => "date",
            ValueType::List => "list",
            ValueType::Dict => "dictionary",
            ValueType::File => "file",
            ValueType::Image => "image",
            ValueType::Url => "url",
            ValueType::Any => "anything",
        };
        f.write_str(s)
    }
}

impl Value {
    pub fn kind(&self) -> ValueType {
        match self {
            Value::Nothing => ValueType::Nothing,
            Value::Text(_) => ValueType::Text,
            Value::Number(_) => ValueType::Number,
            Value::Bool(_) => ValueType::Bool,
            Value::Date(_) => ValueType::Date,
            Value::List(_) => ValueType::List,
            Value::Dict(_) => ValueType::Dict,
            Value::File(_) => ValueType::File,
            Value::Image(_) => ValueType::Image,
            Value::Url(_) => ValueType::Url,
        }
    }

    pub fn text(s: impl Into<String>) -> Self {
        Value::Text(s.into())
    }

    pub fn is_nothing(&self) -> bool {
        matches!(self, Value::Nothing)
    }

    pub fn to_text(&self) -> String {
        match self {
            Value::Nothing => String::new(),
            Value::Text(s) | Value::Url(s) => s.clone(),
            Value::Number(n) => format_number(*n),
            Value::Bool(b) => b.to_string(),
            Value::Date(d) => d.format("%Y-%m-%d %H:%M:%S").to_string(),
            Value::List(items) => items
                .iter()
                .map(Value::to_text)
                .collect::<Vec<_>>()
                .join("\n"),
            Value::Dict(_) => serde_json::to_string_pretty(&self.to_json()).unwrap_or_default(),
            Value::File(p) | Value::Image(p) => p.display().to_string(),
        }
    }

    pub fn to_number(&self) -> Result<f64, ConversionError> {
        let fail = || ConversionError {
            from: self.kind(),
            to: ValueType::Number,
        };
        match self {
            Value::Number(n) => Ok(*n),
            Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Value::Text(s) => parse_number(s).ok_or_else(fail),
            Value::Date(d) => Ok(d.timestamp() as f64),
            Value::List(items) if items.len() == 1 => items[0].to_number(),
            _ => Err(fail()),
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Value::Nothing => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::Text(s) => {
                let s = s.trim().to_lowercase();
                !matches!(s.as_str(), "" | "0" | "false" | "no" | "non" | "off")
            }
            Value::List(items) => !items.is_empty(),
            Value::Dict(map) => !map.is_empty(),
            _ => true,
        }
    }

    pub fn into_list(self) -> Vec<Value> {
        match self {
            Value::List(items) => items,
            Value::Nothing => Vec::new(),
            other => vec![other],
        }
    }

    /// Converts to the requested type. Text is the universal fallback: anything becomes text,
    /// and text is parsed back into numbers, dates, urls or dictionaries (as JSON) on demand.
    pub fn coerce(self, to: ValueType) -> Result<Value, ConversionError> {
        let from = self.kind();
        if to == ValueType::Any || to == from {
            return Ok(self);
        }
        let fail = ConversionError { from, to };
        Ok(match to {
            ValueType::Any => unreachable!(),
            ValueType::Nothing => Value::Nothing,
            ValueType::Text => Value::Text(self.to_text()),
            ValueType::Number => Value::Number(self.to_number().map_err(|_| fail)?),
            ValueType::Bool => Value::Bool(self.to_bool()),
            ValueType::List => Value::List(self.into_list()),
            ValueType::Date => match &self {
                Value::Text(s) => Value::Date(parse_date(s).ok_or(fail)?),
                _ => return Err(fail),
            },
            ValueType::Dict => match self {
                Value::Text(s) => match serde_json::from_str::<serde_json::Value>(&s) {
                    Ok(json @ serde_json::Value::Object(_)) => Value::from_json(json),
                    _ => return Err(fail),
                },
                _ => return Err(fail),
            },
            ValueType::Url => match self {
                Value::Text(s) if looks_like_url(&s) => Value::Url(s.trim().to_string()),
                _ => return Err(fail),
            },
            ValueType::File => match self {
                Value::Text(s) if !s.trim().is_empty() => Value::File(PathBuf::from(s.trim())),
                Value::Image(p) => Value::File(p),
                _ => return Err(fail),
            },
            ValueType::Image => match self {
                Value::File(p) => Value::Image(p),
                _ => return Err(fail),
            },
        })
    }

    pub fn from_json(json: serde_json::Value) -> Value {
        match json {
            serde_json::Value::Null => Value::Nothing,
            serde_json::Value::Bool(b) => Value::Bool(b),
            serde_json::Value::Number(n) => Value::Number(n.as_f64().unwrap_or_default()),
            serde_json::Value::String(s) => Value::Text(s),
            serde_json::Value::Array(items) => {
                Value::List(items.into_iter().map(Value::from_json).collect())
            }
            serde_json::Value::Object(map) => Value::Dict(
                map.into_iter()
                    .map(|(k, v)| (k, Value::from_json(v)))
                    .collect(),
            ),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        use serde_json::Value as J;
        match self {
            Value::Nothing => J::Null,
            Value::Bool(b) => J::Bool(*b),
            // keep integers integral so `{"n": 1}` survives a round trip unchanged
            Value::Number(n) if n.fract() == 0.0 && n.abs() < 9e15 => J::from(*n as i64),
            Value::Number(n) => serde_json::Number::from_f64(*n)
                .map(J::Number)
                .unwrap_or(J::Null),
            Value::List(items) => J::Array(items.iter().map(Value::to_json).collect()),
            Value::Dict(map) => {
                J::Object(map.iter().map(|(k, v)| (k.clone(), v.to_json())).collect())
            }
            Value::Date(d) => J::String(d.to_rfc3339()),
            other => J::String(other.to_text()),
        }
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Text(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Text(s)
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

// accepts french decimal commas, "3,5" is a number for half the users
fn parse_number(s: &str) -> Option<f64> {
    let s = s.trim();
    s.parse::<f64>()
        .ok()
        .or_else(|| s.replace(',', ".").parse::<f64>().ok())
        .filter(|n| n.is_finite())
}

fn parse_date(s: &str) -> Option<DateTime<FixedOffset>> {
    let s = s.trim();
    if let Ok(d) = DateTime::parse_from_rfc3339(s) {
        return Some(d);
    }
    let local_offset = *chrono::Local::now().offset();
    for fmt in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M", "%d/%m/%Y %H:%M"] {
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
            return naive.and_local_timezone(local_offset).single();
        }
    }
    for fmt in ["%Y-%m-%d", "%d/%m/%Y"] {
        if let Ok(day) = chrono::NaiveDate::parse_from_str(s, fmt) {
            return day
                .and_hms_opt(0, 0, 0)?
                .and_local_timezone(local_offset)
                .single();
        }
    }
    None
}

fn looks_like_url(s: &str) -> bool {
    let s = s.trim();
    match s.split_once("://") {
        Some((scheme, rest)) => {
            !rest.is_empty()
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
        }
        None => s.starts_with("mailto:") || s.starts_with("ms-settings:"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_render_without_trailing_zero() {
        assert_eq!(Value::Number(3.0).to_text(), "3");
        assert_eq!(Value::Number(2.5).to_text(), "2.5");
    }

    #[test]
    fn text_parses_to_number_with_comma() {
        assert_eq!(Value::text("3,5").to_number(), Ok(3.5));
        assert_eq!(Value::text(" 42 ").to_number(), Ok(42.0));
        assert!(Value::text("abc").to_number().is_err());
        assert!(Value::text("NaN").to_number().is_err());
    }

    #[test]
    fn truthiness() {
        assert!(!Value::Nothing.to_bool());
        assert!(!Value::text("non").to_bool());
        assert!(!Value::text("").to_bool());
        assert!(Value::text("oui").to_bool());
        assert!(!Value::List(vec![]).to_bool());
        assert!(Value::Number(-1.0).to_bool());
    }

    #[test]
    fn list_joins_lines_as_text() {
        let list = Value::List(vec!["a".into(), 2.0.into()]);
        assert_eq!(list.to_text(), "a\n2");
    }

    #[test]
    fn coerce_text_to_dict_reads_json() {
        let v = Value::text(r#"{"a": 1, "b": [true]}"#)
            .coerce(ValueType::Dict)
            .unwrap();
        let Value::Dict(map) = v else {
            panic!("expected dict")
        };
        assert_eq!(map["a"], Value::Number(1.0));
        assert_eq!(map["b"], Value::List(vec![Value::Bool(true)]));
        assert!(Value::text("[1]").coerce(ValueType::Dict).is_err());
    }

    #[test]
    fn coerce_url_requires_scheme() {
        assert_eq!(
            Value::text("https://example.com").coerce(ValueType::Url),
            Ok(Value::Url("https://example.com".into()))
        );
        assert!(Value::text("example").coerce(ValueType::Url).is_err());
    }

    #[test]
    fn coerce_date_from_text() {
        let v = Value::text("2026-03-01 08:30")
            .coerce(ValueType::Date)
            .unwrap();
        let Value::Date(d) = v else {
            panic!("expected date")
        };
        assert_eq!(d.format("%H:%M").to_string(), "08:30");
    }

    #[test]
    fn any_and_same_type_pass_through() {
        let v = Value::Bool(true);
        assert_eq!(v.clone().coerce(ValueType::Any), Ok(v.clone()));
        assert_eq!(v.clone().coerce(ValueType::Bool), Ok(v));
    }

    #[test]
    fn json_roundtrip_keeps_order() {
        let json: serde_json::Value = serde_json::from_str(r#"{"z": 1, "a": null}"#).unwrap();
        let v = Value::from_json(json.clone());
        assert_eq!(v.to_json(), json);
    }

    #[test]
    fn serde_shape_is_tagged() {
        let s = serde_json::to_string(&Value::text("hi")).unwrap();
        assert_eq!(s, r#"{"type":"text","value":"hi"}"#);
    }
}
