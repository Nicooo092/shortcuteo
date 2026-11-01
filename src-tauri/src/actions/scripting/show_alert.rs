use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct ShowAlert;

#[async_trait]
impl Action for ShowAlert {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "scripting.show_alert",
            Category::Scripting,
            "triangle-alert",
            l("Show alert", "Afficher l'alerte"),
        )
        .describe(
            "Shows a message with OK and Cancel. Cancel stops the shortcut.",
            "Affiche un message avec OK et Annuler. Annuler arrête le raccourci.",
        )
        .sentence("Show alert {message}", "Afficher l'alerte {message}")
        .param(ParamSpec::text("message", "Message", "Message").from_input())
        .param(ParamSpec::text("title", "Title", "Titre").optional())
        .input(ValueType::Any)
        .output(ValueType::Any)
        .unbounded()
    }

    async fn run(
        &self,
        ctx: &mut RunContext,
        input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        match ctx
            .host
            .alert(&params.text("title"), &params.text("message"))
            .await?
        {
            Some(()) => Ok(input),
            None => Err(ActionError::Stop(Value::Nothing)),
        }
    }
}

register_action!(ShowAlert);
