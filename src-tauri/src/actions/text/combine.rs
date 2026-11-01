use async_trait::async_trait;

use super::separator;
use crate::engine::*;
use crate::register_action;

pub struct Combine;

#[async_trait]
impl Action for Combine {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "text.combine",
            Category::Text,
            "merge",
            l("Combine text", "Combiner le texte"),
        )
        .describe(
            "Joins the items of a list into one text.",
            "Assemble les éléments d'une liste en un seul texte.",
        )
        .sentence(
            "Combine {items} with {separator}",
            "Combiner {items} avec {separator}",
        )
        .param(separator::param("newline"))
        .param(separator::custom_param())
        .param(
            ParamSpec::any("items", "Items", "Éléments")
                .from_input()
                .optional(),
        )
        .input(ValueType::List)
        .output(ValueType::Text)
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let sep = separator::resolve(&params);
        let parts: Vec<String> = params
            .value("items")
            .into_list()
            .iter()
            .map(Value::to_text)
            .collect();
        Ok(Value::Text(parts.join(&sep)))
    }
}

register_action!(Combine);
