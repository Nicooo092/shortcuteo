use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct AskInput;

#[async_trait]
impl Action for AskInput {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "scripting.ask_input",
            Category::Scripting,
            "text-cursor-input",
            l("Ask for input", "Demander une entrée"),
        )
        .describe(
            "Asks the user to type something. Dismissing the prompt stops the shortcut.",
            "Demande à l'utilisateur de saisir quelque chose. Fermer la demande arrête le raccourci.",
        )
        .sentence("Ask for {prompt}", "Demander {prompt}")
        .param(ParamSpec::text("prompt", "Prompt", "Question").default(""))
        .param(ParamSpec::text("default", "Default answer", "Réponse par défaut").optional())
        .output(ValueType::Text)
        .unbounded()
    }

    async fn run(
        &self,
        ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        match ctx
            .host
            .ask(&params.text("prompt"), &params.text("default"))
            .await?
        {
            Some(answer) => Ok(Value::Text(answer)),
            None => Err(ActionError::Stop(Value::Nothing)),
        }
    }
}

register_action!(AskInput);
