use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct RepeatEach;

#[async_trait]
impl Action for RepeatEach {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "control.repeat_each",
            Category::ControlFlow,
            "list-restart",
            l("Repeat with each", "Répéter pour chaque élément"),
        )
        .describe(
            "Runs the inner actions once per item. {{item}} and {{index}} hold the current one.",
            "Exécute les actions internes une fois par élément. {{item}} et {{index}} contiennent l'élément en cours.",
        )
        .sentence("Repeat with each item in {items}", "Répéter pour chaque élément de {items}")
        .param(ParamSpec::any("items", "Items", "Éléments").from_input().optional())
        .input(ValueType::List)
        .output(ValueType::List)
        .block(BlockKind::RepeatEach)
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        Ok(Value::List(params.value("items").into_list()))
    }
}

register_action!(RepeatEach);
