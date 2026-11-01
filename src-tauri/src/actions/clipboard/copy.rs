use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct Copy;

#[async_trait]
impl Action for Copy {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "clipboard.copy",
            Category::Clipboard,
            "clipboard-copy",
            l("Copy to clipboard", "Copier dans le presse-papiers"),
        )
        .describe(
            "Puts the content on the clipboard as text.",
            "Place le contenu dans le presse-papiers, en texte.",
        )
        .sentence("Copy {content} to clipboard", "Copier {content}")
        .param(
            ParamSpec::any("content", "Content", "Contenu")
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
        let content = params.value("content");
        ctx.host.clipboard_set(&content.to_text()).await?;
        Ok(content)
    }
}

register_action!(Copy);
