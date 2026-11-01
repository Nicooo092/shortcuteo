use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_notification::NotificationExt;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use ts_rs::TS;

use crate::engine::{ActionError, ActionMeta, Engine, Host, RunError, RunEvent, Value};
use crate::storage::library::Library;
use crate::storage::workflow::{self, Workflow};

pub struct AppState {
    pub engine: Engine,
    pub library: Library,
    runs: Mutex<HashMap<String, CancellationToken>>,
    pending: Mutex<HashMap<u64, oneshot::Sender<serde_json::Value>>>,
    next_request: AtomicU64,
}

impl AppState {
    pub fn new(engine: Engine, library: Library) -> Self {
        AppState {
            engine,
            library,
            runs: Mutex::default(),
            pending: Mutex::default(),
            next_request: AtomicU64::new(1),
        }
    }
}

// answered by the window through host_reply
#[derive(Debug, Clone, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum HostRequest {
    Alert {
        title: String,
        message: String,
    },
    Ask {
        prompt: String,
        default: String,
    },
    Choose {
        prompt: String,
        options: Vec<String>,
        multiple: bool,
    },
    Show {
        value: Value,
    },
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HostRequestEvent {
    pub id: u64,
    pub run_id: String,
    pub request: HostRequest,
}

#[derive(Debug, Clone, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunEventPayload {
    pub run_id: String,
    pub event: RunEvent,
}

struct TauriHost {
    app: AppHandle,
    run_id: String,
}

impl TauriHost {
    async fn request(&self, request: HostRequest) -> Result<serde_json::Value, ActionError> {
        let state = self.app.state::<AppState>();
        let id = state.next_request.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = oneshot::channel();
        state.pending.lock().unwrap().insert(id, tx);
        let event = HostRequestEvent {
            id,
            run_id: self.run_id.clone(),
            request,
        };
        self.app
            .emit("host-request", event)
            .map_err(|e| ActionError::failed(e.to_string()))?;
        // sender dropped means the window went away, treat it like a dismissed dialog
        Ok(rx.await.unwrap_or(serde_json::Value::Null))
    }
}

#[async_trait]
impl Host for TauriHost {
    async fn alert(&self, title: &str, message: &str) -> Result<Option<()>, ActionError> {
        let answer = self
            .request(HostRequest::Alert {
                title: title.into(),
                message: message.into(),
            })
            .await?;
        Ok(answer.as_bool().unwrap_or(false).then_some(()))
    }

    async fn ask(&self, prompt: &str, default: &str) -> Result<Option<String>, ActionError> {
        let answer = self
            .request(HostRequest::Ask {
                prompt: prompt.into(),
                default: default.into(),
            })
            .await?;
        Ok(answer.as_str().map(str::to_string))
    }

    async fn choose(
        &self,
        prompt: &str,
        options: &[String],
        multiple: bool,
    ) -> Result<Option<Vec<usize>>, ActionError> {
        let answer = self
            .request(HostRequest::Choose {
                prompt: prompt.into(),
                options: options.to_vec(),
                multiple,
            })
            .await?;
        Ok(serde_json::from_value::<Vec<usize>>(answer)
            .ok()
            .filter(|v| !v.is_empty()))
    }

    async fn show(&self, value: &Value) -> Result<(), ActionError> {
        self.request(HostRequest::Show {
            value: value.clone(),
        })
        .await?;
        Ok(())
    }

    async fn notify(&self, title: &str, body: &str) -> Result<(), ActionError> {
        self.app
            .notification()
            .builder()
            .title(title)
            .body(body)
            .show()
            .map_err(|e| ActionError::failed(format!("notification failed: {e}")))
    }

    async fn clipboard_get(&self) -> Result<String, ActionError> {
        // an empty or image-only clipboard is an error for the plugin, not for us
        Ok(self.app.clipboard().read_text().unwrap_or_default())
    }

    async fn clipboard_set(&self, text: &str) -> Result<(), ActionError> {
        self.app
            .clipboard()
            .write_text(text)
            .map_err(|e| ActionError::failed(format!("clipboard: {e}")))
    }

    fn find_shortcut(&self, name: &str) -> Option<Workflow> {
        self.app.state::<AppState>().library.find_by_name(name)
    }

    fn shortcut_names(&self) -> Vec<String> {
        self.app
            .state::<AppState>()
            .library
            .list()
            .into_iter()
            .map(|w| w.name)
            .collect()
    }

    fn on_event(&self, event: RunEvent) {
        let payload = RunEventPayload {
            run_id: self.run_id.clone(),
            event,
        };
        let _ = self.app.emit("run-event", payload);
    }
}

#[tauri::command]
pub fn list_actions(state: State<'_, AppState>) -> Vec<ActionMeta> {
    state.engine.registry().metas()
}

#[tauri::command]
pub async fn run_workflow(
    app: AppHandle,
    state: State<'_, AppState>,
    run_id: String,
    workflow: Workflow,
    input: Option<Value>,
) -> Result<Value, RunError> {
    let cancel = CancellationToken::new();
    state
        .runs
        .lock()
        .unwrap()
        .insert(run_id.clone(), cancel.clone());
    let host = Arc::new(TauriHost {
        app,
        run_id: run_id.clone(),
    });
    let result = state
        .engine
        .run(&workflow, input.unwrap_or(Value::Nothing), host, cancel)
        .await;
    state.runs.lock().unwrap().remove(&run_id);
    result
}

#[tauri::command]
pub fn cancel_run(state: State<'_, AppState>, run_id: String) {
    if let Some(token) = state.runs.lock().unwrap().get(&run_id) {
        token.cancel();
    }
}

#[tauri::command]
pub fn host_reply(state: State<'_, AppState>, id: u64, answer: serde_json::Value) {
    if let Some(tx) = state.pending.lock().unwrap().remove(&id) {
        let _ = tx.send(answer);
    }
}

#[tauri::command]
pub fn list_shortcuts(state: State<'_, AppState>) -> Vec<Workflow> {
    state.library.list()
}

#[tauri::command]
pub fn save_shortcut(state: State<'_, AppState>, workflow: Workflow) -> Result<(), String> {
    state.library.save(&workflow).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_shortcut(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.library.delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn parse_shortcut(text: String) -> Result<Workflow, String> {
    workflow::parse(&text).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_shortcut(workflow: Workflow) -> String {
    workflow::to_string(&workflow)
}

#[tauri::command]
pub fn on_battery() -> bool {
    crate::sys::power::on_battery()
}
