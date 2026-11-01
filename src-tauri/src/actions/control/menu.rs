use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct ChooseFromMenu;

#[async_trait]
impl Action for ChooseFromMenu {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "control.menu",
            Category::ControlFlow,
            "list",
            l("Choose from menu", "Choisir dans le menu"),
        )
        .describe(
            "Shows a menu and runs the actions under the chosen entry.",
            "Affiche un menu et exécute les actions sous l'entrée choisie.",
        )
        .sentence("Choose from menu {prompt}", "Choisir dans le menu {prompt}")
        .param(ParamSpec::text("prompt", "Prompt", "Question").optional())
        .input(ValueType::Any)
        .output(ValueType::Any)
        .block(BlockKind::Menu)
        .unbounded()
    }

    // the interpreter shows the menu itself, see BlockKind::Menu
    async fn run(
        &self,
        _ctx: &mut RunContext,
        input: Value,
        _params: Params,
    ) -> Result<Value, ActionError> {
        Ok(input)
    }
}

register_action!(ChooseFromMenu);
