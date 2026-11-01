mod run_command;
mod run_powershell;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use crate::engine::testing::{run, run_with, MockHost};
    use crate::engine::Value;

    #[tokio::test]
    async fn powershell_gets_input_and_returns_json() {
        let out = run_with(
            Arc::new(MockHost::default()),
            Value::Nothing,
            json!([
                { "uuid": "t", "action": "text.text", "params": { "text": "a,b" } },
                { "uuid": "p", "action": "windows.powershell",
                  "params": { "script": "$ShortcutInput.Split(',') | ConvertTo-Json -Compress", "json": true } },
            ]),
        )
        .await
        .unwrap();
        assert_eq!(out, Value::List(vec!["a".into(), "b".into()]));
    }

    #[tokio::test]
    async fn failing_command_reports_exit_code() {
        let err = run(json!([
            { "uuid": "c", "action": "windows.command", "params": { "command": "exit /b 4" } }
        ]))
        .await
        .unwrap_err();
        assert_eq!(err.step.as_deref(), Some("c"));
        assert!(err.message.starts_with("exit code 4"), "{}", err.message);
    }
}
