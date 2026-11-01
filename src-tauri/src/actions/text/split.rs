use async_trait::async_trait;

use super::separator;
use crate::engine::*;
use crate::register_action;

pub struct Split;

#[async_trait]
impl Action for Split {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "text.split",
            Category::Text,
            "split",
            l("Split text", "Scinder le texte"),
        )
        .describe(
            "Cuts the text into a list. \"Every character\" splits it letter by letter.",
            "Découpe le texte en liste. « Chaque caractère » le découpe lettre par lettre.",
        )
        .sentence(
            "Split {text} by {separator}",
            "Scinder {text} par {separator}",
        )
        .param(separator::param("newline"))
        .param(separator::custom_param())
        .param(
            ParamSpec::text("text", "Text", "Texte")
                .from_input()
                .optional(),
        )
        .input(ValueType::Text)
        .output(ValueType::List)
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let text = params.text("text");
        let parts: Vec<Value> = if params.text("separator") == "chars" {
            text.chars().map(|c| Value::Text(c.to_string())).collect()
        } else if params.text("separator") == "newline" {
            text.lines().map(Value::from).collect()
        } else {
            let sep = separator::resolve(&params);
            if sep.is_empty() {
                vec![Value::Text(text)]
            } else {
                text.split(sep.as_str()).map(Value::from).collect()
            }
        };
        Ok(Value::List(parts))
    }
}

register_action!(Split);
