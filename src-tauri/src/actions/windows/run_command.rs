use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;
use crate::sys::process;

pub struct RunCommand;

#[async_trait]
impl Action for RunCommand {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "windows.command",
            Category::Windows,
            "square-terminal",
            l("Run command", "Exécuter une commande"),
        )
        .describe(
            "Runs a command line through cmd.exe and outputs what it prints. The input is sent on stdin.",
            "Exécute une ligne de commande avec cmd.exe et génère ce qu'elle affiche. L'entrée est envoyée sur stdin.",
        )
        .sentence("Run {command}", "Exécuter {command}")
        .param(ParamSpec::text("command", "Command", "Commande"))
        .param(ParamSpec::any("input", "Input", "Entrée").from_input().optional())
        .input(ValueType::Any)
        .output(ValueType::Text)
        .sensitive()
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let out = process::cmd(&params.text("command"), &params.value("input").to_text())
            .await
            .and_then(process::Output::into_result)
            .map_err(ActionError::Failed)?;
        Ok(Value::Text(out))
    }
}

register_action!(RunCommand);
