use async_trait::async_trait;
use serde::Serialize;
use ts_rs::TS;

use super::error::ActionError;
use super::value::Value;
use crate::storage::workflow::Workflow;

/// What a run needs from the outside world: dialogs, notifications, clipboard, other shortcuts.
///
/// The app implements it on top of the window and Tauri plugins; tests use an in-memory one.
/// Dialog methods return `None` when the user dismisses them, which stops the shortcut.
#[async_trait]
pub trait Host: Send + Sync {
    async fn alert(&self, title: &str, message: &str) -> Result<Option<()>, ActionError>;
    async fn ask(&self, prompt: &str, default: &str) -> Result<Option<String>, ActionError>;
    async fn choose(
        &self,
        prompt: &str,
        options: &[String],
        multiple: bool,
    ) -> Result<Option<Vec<usize>>, ActionError>;
    async fn show(&self, value: &Value) -> Result<(), ActionError>;
    async fn notify(&self, title: &str, body: &str) -> Result<(), ActionError>;
    async fn clipboard_get(&self) -> Result<String, ActionError>;
    async fn clipboard_set(&self, text: &str) -> Result<(), ActionError>;
    fn find_shortcut(&self, name: &str) -> Option<Workflow>;
    fn shortcut_names(&self) -> Vec<String>;
    fn on_event(&self, _event: RunEvent) {}
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
#[ts(export)]
pub enum RunEvent {
    StepStarted { step: String },
    StepFinished { step: String, output: Value },
    StepFailed { step: String, message: String },
}
