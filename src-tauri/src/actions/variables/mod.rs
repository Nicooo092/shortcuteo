mod add_to_variable;
mod get_variable;
mod set_variable;

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::engine::testing::run;
    use crate::engine::Value;

    #[tokio::test]
    async fn set_get_and_append() {
        let out = run(json!([
            { "uuid": "a", "action": "variables.set", "params": { "name": "fruits", "value": "apple" } },
            { "uuid": "b", "action": "variables.add", "params": { "name": "fruits", "value": "pear" } },
            { "uuid": "c", "action": "scripting.nothing" },
            { "uuid": "d", "action": "variables.get", "params": { "name": "fruits" } },
        ]))
        .await
        .unwrap();
        assert_eq!(out, Value::List(vec!["apple".into(), "pear".into()]));
    }

    #[tokio::test]
    async fn set_takes_previous_output_and_token_reads_it() {
        let out = run(json!([
            { "uuid": "a", "action": "scripting.count", "params": { "content": [1, 2] } },
            { "uuid": "b", "action": "variables.set", "params": { "name": "n" } },
            { "uuid": "c", "action": "scripting.output", "params": { "value": "n={{var:n}}" } },
        ]))
        .await
        .unwrap();
        assert_eq!(out, Value::text("n=2"));
    }
}
