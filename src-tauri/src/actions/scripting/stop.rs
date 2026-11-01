use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct Stop;

#[async_trait]
impl Action for Stop {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "scripting.stop",
            Category::Scripting,
            "octagon-x",
            l("Stop this shortcut", "Arrêter ce raccourci"),
        )
        .describe("Ends the shortcut here.", "Termine le raccourci ici.")
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        _params: Params,
    ) -> Result<Value, ActionError> {
        Err(ActionError::Stop(Value::Nothing))
    }
}

register_action!(Stop);
