use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct If;

#[async_trait]
impl Action for If {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new("control.if", Category::ControlFlow, "split", l("If", "Si"))
            .describe(
                "Runs the first branch when the condition holds, the Otherwise branch when not.",
                "Exécute la première branche si la condition est vraie, sinon la branche Sinon.",
            )
            .sentence("If {input} {op} {operand}", "Si {input} {op} {operand}")
            .param(
                ParamSpec::any("input", "Input", "Entrée")
                    .from_input()
                    .optional(),
            )
            .param(
                ParamSpec::choice(
                    "op",
                    "Condition",
                    "Condition",
                    vec![
                        choice("is", "is", "est"),
                        choice("is_not", "is not", "n'est pas"),
                        choice("contains", "contains", "contient"),
                        choice("not_contains", "does not contain", "ne contient pas"),
                        choice("begins_with", "begins with", "commence par"),
                        choice("ends_with", "ends with", "finit par"),
                        choice("gt", "is greater than", "est supérieur à"),
                        choice("lt", "is less than", "est inférieur à"),
                        choice("has_value", "has any value", "a une valeur"),
                        choice("no_value", "does not have any value", "n'a aucune valeur"),
                    ],
                )
                .default("has_value"),
            )
            .param(ParamSpec::any("operand", "Value", "Valeur").optional())
            .input(ValueType::Any)
            .output(ValueType::Any)
            .block(BlockKind::If)
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        _input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let lhs = params.value("input");
        let rhs = params.value("operand");
        Ok(Value::Bool(evaluate(&params.text("op"), &lhs, &rhs)))
    }
}

fn evaluate(op: &str, lhs: &Value, rhs: &Value) -> bool {
    let numbers = || Some((lhs.to_number().ok()?, rhs.to_number().ok()?));
    let (a, b) = (lhs.to_text(), rhs.to_text());
    // text comparisons ignore case, like a user would expect from "is"
    let (al, bl) = (a.to_lowercase(), b.to_lowercase());
    match op {
        "is" => numbers().map_or(al == bl, |(x, y)| x == y),
        "is_not" => !evaluate("is", lhs, rhs),
        "contains" => match lhs {
            Value::List(items) => items.iter().any(|i| i.to_text().to_lowercase() == bl),
            _ => al.contains(&bl),
        },
        "not_contains" => !evaluate("contains", lhs, rhs),
        "begins_with" => al.starts_with(&bl),
        "ends_with" => al.ends_with(&bl),
        "gt" => numbers().is_some_and(|(x, y)| x > y),
        "lt" => numbers().is_some_and(|(x, y)| x < y),
        "no_value" => !has_value(lhs),
        _ => has_value(lhs),
    }
}

fn has_value(v: &Value) -> bool {
    match v {
        Value::Nothing => false,
        Value::Text(s) => !s.trim().is_empty(),
        Value::List(items) => !items.is_empty(),
        Value::Dict(map) => !map.is_empty(),
        _ => true,
    }
}

register_action!(If);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_numbers_numerically() {
        assert!(evaluate("is", &Value::text("3.0"), &Value::Number(3.0)));
        assert!(evaluate("gt", &Value::text("10"), &Value::text("9")));
        assert!(!evaluate("gt", &Value::text("abc"), &Value::text("9")));
    }

    #[test]
    fn text_is_case_insensitive() {
        assert!(evaluate("is", &Value::text("Hello"), &Value::text("hello")));
        assert!(evaluate(
            "begins_with",
            &Value::text("Shortcuteo"),
            &Value::text("short")
        ));
    }

    #[test]
    fn contains_checks_list_items() {
        let list = Value::List(vec!["a".into(), "bc".into()]);
        assert!(evaluate("contains", &list, &Value::text("BC")));
        assert!(!evaluate("contains", &list, &Value::text("b")));
    }

    #[test]
    fn blank_text_has_no_value() {
        assert!(evaluate("no_value", &Value::text("  "), &Value::Nothing));
        assert!(evaluate("has_value", &Value::Number(0.0), &Value::Nothing));
    }
}
