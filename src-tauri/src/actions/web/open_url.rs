use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;
use crate::sys::shell;

pub struct OpenUrl;

#[async_trait]
impl Action for OpenUrl {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "web.open_url",
            Category::Web,
            "globe",
            l("Open URL", "Ouvrir l'URL"),
        )
        .describe(
            "Opens a web page or a protocol link (mailto:, ms-settings:, spotify:...) in its default app.",
            "Ouvre une page web ou un lien de protocole (mailto:, ms-settings:, spotify:...) dans son app par défaut.",
        )
        .sentence("Open {url}", "Ouvrir {url}")
        .param(ParamSpec::text("url", "URL", "URL").from_input())
        .input(ValueType::Url)
        .output(ValueType::Url)
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let url = Value::text(params.text("url"))
            .coerce(ValueType::Url)
            .map_err(|_| ActionError::failed(format!("'{}' is not a url", params.text("url"))))?;
        let target = url.to_text();
        tokio::task::spawn_blocking(move || shell::open(&target, ""))
            .await
            .map_err(|e| ActionError::failed(e.to_string()))?
            .map_err(ActionError::Failed)?;
        Ok(url)
    }
}

register_action!(OpenUrl);
