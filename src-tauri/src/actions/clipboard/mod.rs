mod copy;
mod get_clipboard;
mod notify;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use serde_json::json;

    use crate::engine::testing::{run_with, MockHost};
    use crate::engine::Value;

    #[tokio::test]
    async fn copy_then_read_back_and_notify() {
        let host = Arc::new(MockHost::default());
        let out = run_with(
            host.clone(),
            Value::Nothing,
            json!([
                { "uuid": "c", "action": "clipboard.copy", "params": { "content": ["a", "b"] } },
                { "uuid": "g", "action": "clipboard.get" },
                { "uuid": "n", "action": "notification.show" },
            ]),
        )
        .await
        .unwrap();
        assert_eq!(out, Value::text("a\nb"));
        assert_eq!(
            *host.notifications.lock().unwrap(),
            vec![("Shortcuteo".to_string(), "a\nb".to_string())]
        );
    }
}
