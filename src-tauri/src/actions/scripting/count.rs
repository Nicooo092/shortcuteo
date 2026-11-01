use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct Count;

#[async_trait]
impl Action for Count {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "scripting.count",
            Category::Scripting,
            "hash",
            l("Count", "Compter"),
        )
        .describe(
            "Counts the items, characters, words or lines of the input.",
            "Compte les éléments, caractères, mots ou lignes de l'entrée.",
        )
        .sentence(
            "Count {unit} in {content}",
            "Compter les {unit} dans {content}",
        )
        .param(
            ParamSpec::choice(
                "unit",
                "Unit",
                "Unité",
                vec![
                    choice("items", "items", "éléments"),
                    choice("chars", "characters", "caractères"),
                    choice("words", "words", "mots"),
                    choice("lines", "lines", "lignes"),
                ],
            )
            .default("items"),
        )
        .param(
            ParamSpec::any("content", "Content", "Contenu")
                .from_input()
                .optional(),
        )
        .input(ValueType::Any)
        .output(ValueType::Number)
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let content = params.value("content");
        let n = match params.text("unit").as_str() {
            "chars" => content.to_text().chars().count(),
            "words" => content.to_text().split_whitespace().count(),
            "lines" => content.to_text().lines().count(),
            _ => content.into_list().len(),
        };
        Ok(Value::Number(n as f64))
    }
}

register_action!(Count);
