use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct SetVariable;

#[async_trait]
impl Action for SetVariable {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "variables.set",
            Category::Variables,
            "variable",
            l("Set variable", "Définir la variable"),
        )
        .describe(
            "Stores a value under a name. Read it back with {{var:name}}.",
            "Range une valeur sous un nom. Relisez-la avec {{var:nom}}.",
        )
        .sentence(
            "Set variable {name} to {value}",
            "Définir {name} sur {value}",
        )
        .param(ParamSpec::variable("name", "Variable", "Variable"))
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
        ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let value = params.value("value");
        ctx.set_variable(&params.text("name"), value.clone());
        Ok(value)
    }
}

register_action!(SetVariable);
