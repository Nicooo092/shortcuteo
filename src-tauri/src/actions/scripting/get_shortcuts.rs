use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct GetShortcuts;

#[async_trait]
impl Action for GetShortcuts {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "scripting.get_shortcuts",
            Category::Scripting,
            "library",
            l("Get my shortcuts", "Obtenir mes raccourcis"),
        )
        .describe(
            "Outputs the names of all saved shortcuts.",
            "Génère le nom de tous les raccourcis enregistrés.",
        )
        .output(ValueType::List)
    }

    async fn run(
        &self,
        ctx: &mut RunContext,
        _input: Value,
        _params: Params,
    ) -> Result<Value, ActionError> {
        Ok(Value::List(
            ctx.host
                .shortcut_names()
                .into_iter()
                .map(Value::Text)
                .collect(),
        ))
    }
}

register_action!(GetShortcuts);
