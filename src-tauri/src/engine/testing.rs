use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use tokio_util::sync::CancellationToken;

use super::{ActionError, Engine, EngineConfig, Host, Registry, RunError, RunEvent, Value};
use crate::storage::workflow::{self, Workflow};

pub enum Answer {
    Dismiss,
    Ok,
    Text(&'static str),
    Pick(usize),
}

#[derive(Default)]
pub struct MockHost {
    pub answers: Mutex<VecDeque<Answer>>,
    pub clipboard: Mutex<String>,
    pub notifications: Mutex<Vec<(String, String)>>,
    pub shown: Mutex<Vec<Value>>,
    pub events: Mutex<Vec<RunEvent>>,
    pub shortcuts: HashMap<String, Workflow>,
}

impl MockHost {
    pub fn answering(answers: impl IntoIterator<Item = Answer>) -> Self {
        MockHost {
            answers: Mutex::new(answers.into_iter().collect()),
            ..Default::default()
        }
    }

    pub fn with_shortcut(mut self, name: &str, actions: serde_json::Value) -> Self {
        self.shortcuts
            .insert(name.to_string(), workflow_from(actions));
        self
    }

    fn next(&self) -> Answer {
        self.answers
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or(Answer::Dismiss)
    }

    pub fn started_steps(&self) -> Vec<String> {
        self.events
            .lock()
            .unwrap()
            .iter()
            .filter_map(|e| match e {
                RunEvent::StepStarted { step } => Some(step.clone()),
                _ => None,
            })
            .collect()
    }
}

#[async_trait]
impl Host for MockHost {
    async fn alert(&self, _title: &str, _message: &str) -> Result<Option<()>, ActionError> {
        Ok(match self.next() {
            Answer::Dismiss => None,
            _ => Some(()),
        })
    }

    async fn ask(&self, _prompt: &str, default: &str) -> Result<Option<String>, ActionError> {
        Ok(match self.next() {
            Answer::Text(t) => Some(t.to_string()),
            Answer::Ok => Some(default.to_string()),
            _ => None,
        })
    }

    async fn choose(
        &self,
        _prompt: &str,
        _options: &[String],
        _multiple: bool,
    ) -> Result<Option<Vec<usize>>, ActionError> {
        Ok(match self.next() {
            Answer::Pick(i) => Some(vec![i]),
            _ => None,
        })
    }

    async fn show(&self, value: &Value) -> Result<(), ActionError> {
        self.shown.lock().unwrap().push(value.clone());
        Ok(())
    }

    async fn notify(&self, title: &str, body: &str) -> Result<(), ActionError> {
        self.notifications
            .lock()
            .unwrap()
            .push((title.to_string(), body.to_string()));
        Ok(())
    }

    async fn clipboard_get(&self) -> Result<String, ActionError> {
        Ok(self.clipboard.lock().unwrap().clone())
    }

    async fn clipboard_set(&self, text: &str) -> Result<(), ActionError> {
        *self.clipboard.lock().unwrap() = text.to_string();
        Ok(())
    }

    fn find_shortcut(&self, name: &str) -> Option<Workflow> {
        self.shortcuts.get(name).cloned()
    }

    fn shortcut_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.shortcuts.keys().cloned().collect();
        names.sort();
        names
    }

    fn on_event(&self, event: RunEvent) {
        self.events.lock().unwrap().push(event);
    }
}

pub fn workflow_from(actions: serde_json::Value) -> Workflow {
    let doc = serde_json::json!({
        "schemaVersion": workflow::SCHEMA_VERSION,
        "id": "test",
        "name": "test",
        "actions": actions,
    });
    workflow::parse(&doc.to_string()).expect("valid test workflow")
}

pub fn engine() -> Engine {
    Engine::new(Registry::builtin(), EngineConfig::default())
}

pub async fn run_with(
    host: Arc<MockHost>,
    input: Value,
    actions: serde_json::Value,
) -> Result<Value, RunError> {
    engine()
        .run(
            &workflow_from(actions),
            input,
            host,
            CancellationToken::new(),
        )
        .await
}

pub async fn run(actions: serde_json::Value) -> Result<Value, RunError> {
    run_with(Arc::new(MockHost::default()), Value::Nothing, actions).await
}
