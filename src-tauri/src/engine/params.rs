use indexmap::IndexMap;

use super::error::ActionError;
use super::value::Value;

/// Resolved parameters of a step, keyed like [`ParamSpec::key`](super::action::ParamSpec).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Params(IndexMap<String, Value>);

impl Params {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: impl Into<String>, value: Value) {
        self.0.insert(key.into(), value);
    }

    pub fn with(mut self, key: &str, value: impl Into<Value>) -> Self {
        self.insert(key, value.into());
        self
    }

    pub fn value(&self, key: &str) -> Value {
        self.0.get(key).cloned().unwrap_or(Value::Nothing)
    }

    pub fn has(&self, key: &str) -> bool {
        self.0.get(key).is_some_and(|v| !v.is_nothing())
    }

    pub fn text(&self, key: &str) -> String {
        self.0.get(key).map(Value::to_text).unwrap_or_default()
    }

    pub fn number(&self, key: &str) -> Result<f64, ActionError> {
        match self.0.get(key) {
            None | Some(Value::Nothing) => Err(ActionError::MissingParam(key.to_string())),
            Some(v) => v.to_number().map_err(|source| ActionError::BadParam {
                param: key.to_string(),
                source,
            }),
        }
    }

    pub fn bool(&self, key: &str) -> bool {
        self.0.get(key).is_some_and(Value::to_bool)
    }
}
