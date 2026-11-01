mod change_case;
mod combine;
mod replace;
mod separator;
mod split;
#[allow(clippy::module_inception)]
mod text;
mod trim;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::engine::testing::run;
    use crate::engine::Value;

    #[tokio::test]
    async fn split_then_combine_roundtrips() {
        let out = run(json!([
            { "uuid": "t", "action": "text.text", "params": { "text": "  a, b ,c  " } },
            { "uuid": "trim", "action": "text.trim" },
            { "uuid": "s", "action": "text.split", "params": { "separator": "comma" } },
            { "uuid": "r", "action": "control.repeat_each",
              "body": [ { "uuid": "x", "action": "text.trim" } ] },
            { "uuid": "c", "action": "text.combine",
              "params": { "separator": "custom", "custom": " | " } },
        ]))
        .await
        .unwrap();
        assert_eq!(out, Value::text("a | b | c"));
    }

    #[tokio::test]
    async fn split_by_characters_and_lines() {
        let chars = run(json!([
            { "uuid": "s", "action": "text.split", "params": { "text": "été", "separator": "chars" } }
        ]))
        .await
        .unwrap();
        assert_eq!(chars, Value::List(vec!["é".into(), "t".into(), "é".into()]));

        let lines = run(json!([
            { "uuid": "s", "action": "text.split", "params": { "text": "a\r\nb" } }
        ]))
        .await
        .unwrap();
        assert_eq!(lines, Value::List(vec!["a".into(), "b".into()]));
    }

    #[tokio::test]
    async fn change_case_uses_previous_output() {
        let out = run(json!([
            { "uuid": "t", "action": "text.text", "params": { "text": "hello" } },
            { "uuid": "c", "action": "text.change_case", "params": { "case": "upper" } },
            { "uuid": "r", "action": "text.replace", "params": { "find": "l", "with": "_", "case_sensitive": false } },
        ]))
        .await
        .unwrap();
        assert_eq!(out, Value::text("HE__O"));
    }
}
