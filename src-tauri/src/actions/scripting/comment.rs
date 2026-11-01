use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct Comment;

#[async_trait]
impl Action for Comment {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "scripting.comment",
            Category::Scripting,
            "message-square",
            l("Comment", "Commentaire"),
        )
        .describe(
            "A note for whoever reads the shortcut. Does nothing and lets the previous output through.",
            "Une note pour qui lit le raccourci. Ne fait rien et laisse passer la sortie précédente.",
        )
        .sentence("{text}", "{text}")
        .param(ParamSpec::multiline("text", "Text", "Texte").default(""))
        .input(ValueType::Any)
        .output(ValueType::Any)
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        input: Value,
        _params: Params,
    ) -> Result<Value, ActionError> {
        Ok(input)
    }
}

register_action!(Comment);
