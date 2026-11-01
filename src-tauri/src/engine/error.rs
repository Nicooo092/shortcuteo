use std::time::Duration;

use serde::Serialize;
use ts_rs::TS;

use super::value::{ConversionError, Value};

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ActionError {
    #[error("unknown action '{0}'")]
    UnknownAction(String),
    #[error("missing parameter '{0}'")]
    MissingParam(String),
    #[error("parameter '{param}': {source}")]
    BadParam {
        param: String,
        source: ConversionError,
    },
    #[error("{0}")]
    Failed(String),
    #[error("cancelled")]
    Cancelled,
    #[error("took longer than {}s", .0.as_secs())]
    Timeout(Duration),
    // not a failure: unwinds the run and becomes its output
    #[error("stopped")]
    Stop(Value),
}

impl ActionError {
    pub fn failed(msg: impl Into<String>) -> Self {
        ActionError::Failed(msg.into())
    }
}

/// Error reported for a whole run, pointing at the step that failed.
#[derive(Debug, Clone, PartialEq, Serialize, TS, thiserror::Error)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
#[error("{}", self.describe())]
pub struct RunError {
    pub step: Option<String>,
    pub action: Option<String>,
    pub message: String,
    pub cancelled: bool,
}

impl RunError {
    pub fn at(step: &str, action: &str, error: &ActionError) -> Self {
        RunError {
            step: Some(step.to_string()),
            action: Some(action.to_string()),
            message: error.to_string(),
            cancelled: matches!(error, ActionError::Cancelled),
        }
    }

    pub fn general(message: impl Into<String>) -> Self {
        RunError {
            step: None,
            action: None,
            message: message.into(),
            cancelled: false,
        }
    }

    fn describe(&self) -> String {
        match &self.action {
            Some(action) => format!("{action}: {}", self.message),
            None => self.message.clone(),
        }
    }
}
