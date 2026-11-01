use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct ShowNotification;

#[async_trait]
impl Action for ShowNotification {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "notification.show",
            Category::Clipboard,
            "bell",
            l("Show notification", "Afficher une notification"),
        )
        .describe(
            "Shows a Windows notification.",
            "Affiche une notification Windows.",
        )
        .sentence(
            "Show notification {body}",
            "Afficher la notification {body}",
        )
        .param(ParamSpec::text("body", "Body", "Texte").from_input())
        .param(ParamSpec::text("title", "Title", "Titre").default("Shortcuteo"))
        .input(ValueType::Any)
        .output(ValueType::Any)
    }

    async fn run(
        &self,
        ctx: &mut RunContext,
        input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        ctx.host
            .notify(&params.text("title"), &params.text("body"))
            .await?;
        Ok(input)
    }
}

register_action!(ShowNotification);
