use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct AddToVariable;

#[async_trait]
impl Action for AddToVariable {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "variables.add",
            Category::Variables,
            "list-plus",
            l("Add to variable", "Ajouter à la variable"),
        )
        .describe(
            "Appends a value to a variable, turning it into a list.",
            "Ajoute une valeur à une variable, qui devient une liste.",
        )
        .sentence("Add {value} to {name}", "Ajouter {value} à {name}")
        .param(ParamSpec::variable("name", "Variable", "Variable"))
        .param(
            ParamSpec::any("value", "Value", "Valeur")
                .from_input()
                .optional(),
        )
        .input(ValueType::Any)
        .output(ValueType::List)
    }

    async fn run(
        &self,
        ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let name = params.text("name");
        let mut items = ctx.variable(&name).into_list();
        items.push(params.value("value"));
        let list = Value::List(items);
        ctx.set_variable(&name, list.clone());
        Ok(list)
    }
}

register_action!(AddToVariable);
