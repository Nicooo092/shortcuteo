use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;
use crate::sys::shell;

pub struct OpenApp;

#[async_trait]
impl Action for OpenApp {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "navigation.open_app",
            Category::Navigation,
            "app-window",
            l("Open app", "Ouvrir une app"),
        )
        .describe(
            "Starts an app from its name (notepad, calc), its path, or shell:AppsFolder\\<id> for Store apps.",
            "Lance une app par son nom (notepad, calc), son chemin, ou shell:AppsFolder\\<id> pour les apps du Store.",
        )
        .sentence("Open {app}", "Ouvrir {app}")
        .param(ParamSpec::text("app", "App", "App"))
        .param(ParamSpec::text("args", "Arguments", "Arguments").optional())
        .output(ValueType::Nothing)
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let app = params.text("app");
        let args = params.text("args");
        if app.trim().is_empty() {
            return Err(ActionError::MissingParam("app".to_string()));
        }
        tokio::task::spawn_blocking(move || shell::open(app.trim(), &args))
            .await
            .map_err(|e| ActionError::failed(e.to_string()))?
            .map_err(ActionError::Failed)?;
        Ok(Value::Nothing)
    }
}

register_action!(OpenApp);
