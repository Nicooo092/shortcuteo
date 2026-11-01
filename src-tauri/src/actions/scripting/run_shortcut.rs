use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct RunShortcut;

#[async_trait]
impl Action for RunShortcut {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "scripting.run_shortcut",
            Category::Scripting,
            "workflow",
            l("Run shortcut", "Exécuter un raccourci"),
        )
        .describe(
            "Runs another shortcut and outputs what it outputs.",
            "Exécute un autre raccourci et génère sa sortie.",
        )
        .sentence("Run {name} with {input}", "Exécuter {name} avec {input}")
        .param(ParamSpec::text("name", "Shortcut", "Raccourci"))
        .param(
            ParamSpec::any("input", "Input", "Entrée")
                .from_input()
                .optional(),
        )
        .input(ValueType::Any)
        .output(ValueType::Any)
        .unbounded()
    }

    async fn run(
        &self,
        ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        ctx.run_shortcut(&params.text("name"), params.value("input"))
            .await
    }
}

register_action!(RunShortcut);
