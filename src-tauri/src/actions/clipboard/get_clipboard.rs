use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct GetClipboard;

#[async_trait]
impl Action for GetClipboard {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "clipboard.get",
            Category::Clipboard,
            "clipboard-paste",
            l("Get clipboard", "Obtenir le presse-papiers"),
        )
        .describe(
            "Outputs the text on the clipboard.",
            "Génère le texte du presse-papiers.",
        )
        .output(ValueType::Text)
    }

    async fn run(
        &self,
        ctx: &mut RunContext,
        _input: Value,
        _params: Params,
    ) -> Result<Value, ActionError> {
        Ok(Value::Text(ctx.host.clipboard_get().await?))
    }
}

register_action!(GetClipboard);
