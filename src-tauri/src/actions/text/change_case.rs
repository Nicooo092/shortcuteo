use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct ChangeCase;

#[async_trait]
impl Action for ChangeCase {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "text.change_case",
            Category::Text,
            "case-sensitive",
            l("Change case", "Modifier la casse"),
        )
        .describe(
            "Changes the text to upper case, lower case, title case or sentence case.",
            "Passe le texte en majuscules, minuscules, initiales en majuscule ou phrase.",
        )
        .sentence("Change {text} to {case}", "Passer {text} en {case}")
        .param(
            ParamSpec::choice(
                "case",
                "Case",
                "Casse",
                vec![
                    choice("upper", "UPPERCASE", "MAJUSCULES"),
                    choice("lower", "lowercase", "minuscules"),
                    choice("title", "Title Case", "Initiales En Majuscule"),
                    choice("sentence", "Sentence case", "Phrase"),
                ],
            )
            .default("upper"),
        )
        .param(
            ParamSpec::text("text", "Text", "Texte")
                .from_input()
                .optional(),
        )
        .input(ValueType::Text)
        .output(ValueType::Text)
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let text = params.text("text");
        let out = match params.text("case").as_str() {
            "lower" => text.to_lowercase(),
            "title" => title_case(&text),
            "sentence" => sentence_case(&text),
            _ => text.to_uppercase(),
        };
        Ok(Value::Text(out))
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first
            .to_uppercase()
            .chain(chars.flat_map(char::to_lowercase))
            .collect(),
        None => String::new(),
    }
}

// splits on spaces only so "l'été" and "jean-luc" stay one word
fn title_case(text: &str) -> String {
    text.split(' ')
        .map(capitalize)
        .collect::<Vec<_>>()
        .join(" ")
}

fn sentence_case(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut start = true;
    for c in text.chars() {
        if start && c.is_alphabetic() {
            out.extend(c.to_uppercase());
            start = false;
        } else {
            out.extend(c.to_lowercase());
        }
        if matches!(c, '.' | '!' | '?' | '\n') {
            start = true;
        }
    }
    out
}

register_action!(ChangeCase);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_keeps_apostrophes_and_hyphens() {
        assert_eq!(title_case("l'été à jean-luc"), "L'été À Jean-luc");
    }

    #[test]
    fn sentence_restarts_after_punctuation() {
        assert_eq!(
            sentence_case("HELLO. how ARE you? fine"),
            "Hello. How are you? Fine"
        );
    }
}
