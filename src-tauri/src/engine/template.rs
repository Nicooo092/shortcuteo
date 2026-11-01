use super::context::RunContext;
use super::value::Value;

// a string that is exactly one token keeps its type, so a list stays a list
pub fn render(s: &str, ctx: &RunContext) -> Value {
    render_with(s, |t| ctx.token(t))
}

pub(super) fn render_with(s: &str, lookup: impl Fn(&str) -> Option<Value>) -> Value {
    if let Some(inner) = s.strip_prefix("{{").and_then(|r| r.strip_suffix("}}")) {
        if !inner.contains("{{") && !inner.contains("}}") {
            if let Some(v) = lookup(inner.trim()) {
                return v;
            }
        }
    }
    if !s.contains("{{") {
        return Value::Text(s.to_string());
    }

    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            out.push_str(&rest[start..]);
            rest = "";
            break;
        };
        let token = &after[..end];
        match lookup(token.trim()) {
            Some(v) => out.push_str(&v.to_text()),
            None => out.push_str(&rest[start..start + 2 + end + 2]),
        }
        rest = &after[end + 2..];
    }
    out.push_str(rest);
    Value::Text(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(t: &str) -> Option<Value> {
        match t {
            "var:n" => Some(Value::Number(3.0)),
            "var:list" => Some(Value::List(vec!["a".into(), "b".into()])),
            "input" => Some(Value::Nothing),
            _ => None,
        }
    }

    #[test]
    fn single_token_keeps_type() {
        assert_eq!(
            render_with("{{var:list}}", lookup),
            lookup("var:list").unwrap()
        );
        assert_eq!(render_with("{{ var:n }}", lookup), Value::Number(3.0));
    }

    #[test]
    fn mixed_text_is_interpolated() {
        assert_eq!(render_with("n = {{var:n}}!", lookup), Value::text("n = 3!"));
        assert_eq!(render_with("{{var:n}}{{var:n}}", lookup), Value::text("33"));
        assert_eq!(render_with("[{{input}}]", lookup), Value::text("[]"));
    }

    #[test]
    fn unknown_or_broken_tokens_stay_literal() {
        assert_eq!(render_with("{{nope}} x", lookup), Value::text("{{nope}} x"));
        assert_eq!(render_with("a {{var:n", lookup), Value::text("a {{var:n"));
        assert_eq!(render_with("{{nope}}", lookup), Value::text("{{nope}}"));
    }
}
