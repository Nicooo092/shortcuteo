use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct StopAndOutput;

#[async_trait]
impl Action for StopAndOutput {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "scripting.output",
            Category::Scripting,
            "log-out",
            l("Stop and output", "Arrêter et générer"),
        )
        .describe(
            "Ends the shortcut and hands the value back to whoever ran it.",
            "Termine le raccourci et renvoie la valeur à celui qui l'a lancé.",
        )
        .sentence("Stop and output {value}", "Arrêter et générer {value}")
        .param(
            ParamSpec::any("value", "Value", "Valeur")
                .from_input()
                .optional(),
        )
        .input(ValueType::Any)
        .output(ValueType::Any)
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        Err(ActionError::Stop(params.value("value")))
    }
}

register_action!(StopAndOutput);
