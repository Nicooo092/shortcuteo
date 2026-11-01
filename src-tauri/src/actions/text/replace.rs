use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct Replace;

#[async_trait]
impl Action for Replace {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "text.replace",
            Category::Text,
            "replace",
            l("Replace text", "Remplacer le texte"),
        )
        .describe(
            "Replaces every occurrence of a text by another.",
            "Remplace chaque occurrence d'un texte par un autre.",
        )
        .sentence(
            "Replace {find} with {with} in {text}",
            "Remplacer {find} par {with} dans {text}",
        )
        .param(ParamSpec::text("find", "Find", "Rechercher"))
        .param(ParamSpec::text("with", "Replace with", "Remplacer par").default(""))
        .param(
            ParamSpec::boolean("case_sensitive", "Case sensitive", "Respecter la casse")
                .default(true),
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
        let find = params.text("find");
        let with = params.text("with");
        if find.is_empty() {
            return Ok(Value::Text(text));
        }
        let out = if params.bool("case_sensitive") {
            text.replace(&find, &with)
        } else {
            replace_ignore_case(&text, &find, &with)
        };
        Ok(Value::Text(out))
    }
}

fn replace_ignore_case(text: &str, find: &str, with: &str) -> String {
    let needle: Vec<char> = find.chars().flat_map(char::to_lowercase).collect();
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    // compare char by char: lowercasing the whole string can change byte offsets (e.g. 'İ')
    while i < chars.len() {
        let mut lowered = Vec::with_capacity(needle.len());
        let mut j = i;
        while j < chars.len() && lowered.len() < needle.len() {
            lowered.extend(chars[j].to_lowercase());
            j += 1;
        }
        if lowered == needle {
            out.push_str(with);
            i = j;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

register_action!(Replace);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_case_when_asked() {
        assert_eq!(replace_ignore_case("Foo foo FOO", "foo", "x"), "x x x");
        assert_eq!(replace_ignore_case("Été", "été", "summer"), "summer");
        assert_eq!(replace_ignore_case("abc", "zz", "x"), "abc");
    }
}
