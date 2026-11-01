use std::time::Duration;

use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct Wait;

#[async_trait]
impl Action for Wait {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "scripting.wait",
            Category::Scripting,
            "hourglass",
            l("Wait", "Attendre"),
        )
        .describe("Pauses the shortcut.", "Met le raccourci en pause.")
        .sentence("Wait {seconds}", "Attendre {seconds}")
        .param(
            ParamSpec::number("seconds", "Duration", "Durée")
                .range(0.0, 86_400.0)
                .unit("s")
                .default(1),
        )
        .input(ValueType::Any)
        .output(ValueType::Any)
        .unbounded()
    }

    async fn run(
        &self,
        _ctx: &mut RunContext,
        input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let secs = params.number("seconds")?.clamp(0.0, 86_400.0);
        // stop button works because the interpreter drops this future
        tokio::time::sleep(Duration::from_secs_f64(secs)).await;
        Ok(input)
    }
}

register_action!(Wait);
