use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;
use crate::sys::process;

pub struct RunPowerShell;

#[async_trait]
impl Action for RunPowerShell {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "windows.powershell",
            Category::Windows,
            "terminal",
            l("Run PowerShell script", "Exécuter un script PowerShell"),
        )
        .describe(
            "Runs a script with Windows PowerShell and outputs what it prints. The input is available as $ShortcutInput.",
            "Exécute un script avec Windows PowerShell et génère ce qu'il affiche. L'entrée est disponible dans $ShortcutInput.",
        )
        .sentence("Run PowerShell {script}", "Exécuter PowerShell {script}")
        .param(ParamSpec::multiline("script", "Script", "Script"))
        .param(ParamSpec::any("input", "Input", "Entrée").from_input().optional())
        .param(
            ParamSpec::boolean("json", "Parse output as JSON", "Lire la sortie en JSON")
                .default(false),
        )
        .input(ValueType::Any)
        .output(ValueType::Any)
        .sensitive()
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let stdin = match params.value("input") {
            v @ (Value::Dict(_) | Value::List(_)) => v.to_json().to_string(),
            v => v.to_text(),
        };
        let out = process::powershell(&params.text("script"), &stdin)
            .await
            .and_then(process::Output::into_result)
            .map_err(ActionError::Failed)?;
        if params.bool("json") {
            let json = serde_json::from_str(&out)
                .map_err(|e| ActionError::failed(format!("output is not JSON: {e}")))?;
            return Ok(Value::from_json(json));
        }
        Ok(Value::Text(out))
    }
}

register_action!(RunPowerShell);
