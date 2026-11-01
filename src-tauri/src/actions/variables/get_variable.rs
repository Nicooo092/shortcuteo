use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct GetVariable;

#[async_trait]
impl Action for GetVariable {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "variables.get",
            Category::Variables,
            "variable",
            l("Get variable", "Obtenir la variable"),
        )
        .describe(
            "Outputs the value stored under a name, or nothing if it was never set.",
            "Génère la valeur rangée sous un nom, ou rien si elle n'a jamais été définie.",
        )
        .sentence("Get variable {name}", "Obtenir {name}")
        .param(ParamSpec::variable("name", "Variable", "Variable"))
        .output(ValueType::Any)
    }

    async fn run(
        &self,
        ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        Ok(ctx.variable(&params.text("name")))
    }
}

register_action!(GetVariable);
