use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct Repeat;

#[async_trait]
impl Action for Repeat {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new("control.repeat", Category::ControlFlow, "repeat", l("Repeat", "Répéter"))
            .describe(
                "Runs the inner actions a number of times and outputs the list of their results. {{index}} holds the current turn.",
                "Exécute les actions internes un certain nombre de fois et génère la liste de leurs résultats. {{index}} contient le tour en cours.",
            )
            .sentence("Repeat {count} times", "Répéter {count} fois")
            .param(ParamSpec::number("count", "Times", "Fois").range(0.0, 100_000.0).default(2))
            .input(ValueType::Any)
            .output(ValueType::List)
            .block(BlockKind::Repeat)
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        Ok(Value::Number(params.number("count")?.floor()))
    }
}

register_action!(Repeat);
