use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct Nothing;

#[async_trait]
impl Action for Nothing {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "scripting.nothing",
            Category::Scripting,
            "circle-slash",
            l("Nothing", "Rien"),
        )
        .describe(
            "Outputs nothing, so the next action starts without input.",
            "Ne génère rien, l'action suivante démarre sans entrée.",
        )
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        _params: Params,
    ) -> Result<Value, ActionError> {
        Ok(Value::Nothing)
    }
}

register_action!(Nothing);
