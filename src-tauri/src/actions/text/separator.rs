use crate::engine::{choice, ParamSpec, Params};

pub fn param(default: &str) -> ParamSpec {
    ParamSpec::choice(
        "separator",
        "Separator",
        "Séparateur",
        vec![
            choice("newline", "new lines", "retours à la ligne"),
            choice("space", "spaces", "espaces"),
            choice("comma", "commas", "virgules"),
            choice("chars", "every character", "chaque caractère"),
            choice("custom", "custom", "personnalisé"),
        ],
    )
    .default(default)
}

pub fn custom_param() -> ParamSpec {
    ParamSpec::text("custom", "Custom separator", "Séparateur personnalisé").optional()
}

pub fn resolve(params: &Params) -> String {
    match params.text("separator").as_str() {
        "newline" => "\n".to_string(),
        "space" => " ".to_string(),
        "comma" => ",".to_string(),
        "chars" => String::new(),
        _ => params.text("custom"),
    }
}
