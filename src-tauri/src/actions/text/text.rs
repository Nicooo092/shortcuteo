use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct Text;

#[async_trait]
impl Action for Text {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new("text.text", Category::Text, "type", l("Text", "Texte"))
            .describe(
                "Outputs the text you type, with variables filled in.",
                "Génère le texte saisi, variables remplacées.",
            )
            .sentence("{text}", "{text}")
            .param(ParamSpec::multiline("text", "Text", "Texte").default(""))
            .output(ValueType::Text)
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        Ok(Value::Text(params.text("text")))
    }
}

register_action!(Text);
