use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct Trim;

#[async_trait]
impl Action for Trim {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "text.trim",
            Category::Text,
            "remove-formatting",
            l("Trim whitespace", "Supprimer les espaces"),
        )
        .describe(
            "Removes spaces and line breaks at the start and end of the text.",
            "Retire les espaces et retours à la ligne au début et à la fin du texte.",
        )
        .sentence("Trim {text}", "Nettoyer {text}")
        .param(
            ParamSpec::text("text", "Text", "Texte")
                .from_input()
                .optional(),
        )
        .input(ValueType::Text)
        .output(ValueType::Text)
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        Ok(Value::Text(params.text("text").trim().to_string()))
    }
}

register_action!(Trim);
