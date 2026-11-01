use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct ShowResult;

#[async_trait]
impl Action for ShowResult {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "scripting.show_result",
            Category::Scripting,
            "panel-top-open",
            l("Show content", "Afficher le contenu"),
        )
        .describe(
            "Opens a preview window with the content.",
            "Ouvre une fenêtre d'aperçu avec le contenu.",
        )
        .sentence("Show {content}", "Afficher {content}")
        .param(ParamSpec::any("content", "Content", "Contenu").from_input())
        .input(ValueType::Any)
        .output(ValueType::Any)
    }

    async fn run(
        &self,
        ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let content = params.value("content");
        ctx.host.show(&content).await?;
        Ok(content)
    }
}

register_action!(ShowResult);
