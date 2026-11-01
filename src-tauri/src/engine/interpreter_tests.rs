use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::json;
use tokio_util::sync::CancellationToken;

use super::testing::{run, run_with, workflow_from, Answer, MockHost};
use super::*;

#[tokio::test]
async fn previous_output_flows_into_next_step() {
    let out = run(json!([
        { "uuid": "a", "action": "scripting.count", "params": { "content": ["x", "y", "z"] } },
        { "uuid": "b", "action": "scripting.comment", "params": { "text": "keeps input" } },
    ]))
    .await
    .unwrap();
    assert_eq!(out, Value::Number(3.0));
}

#[tokio::test]
async fn stop_and_output_ends_the_run() {
    let host = Arc::new(MockHost::default());
    let out = run_with(
        host.clone(),
        Value::text("hello"),
        json!([
            { "uuid": "a", "action": "scripting.output", "params": { "value": "{{input}}!" } },
            { "uuid": "b", "action": "scripting.show_result", "params": { "content": "never" } },
        ]),
    )
    .await
    .unwrap();
    assert_eq!(out, Value::text("hello!"));
    assert!(host.shown.lock().unwrap().is_empty());
}

#[tokio::test]
async fn output_token_keeps_type() {
    let out = run(json!([
        { "uuid": "list", "action": "control.repeat_each", "params": { "items": ["a", "b"] } },
        { "uuid": "n", "action": "scripting.count", "params": { "content": "{{output:list}}" } },
    ]))
    .await
    .unwrap();
    assert_eq!(out, Value::Number(2.0));
}

#[tokio::test]
async fn if_picks_branch_from_shortcut_input() {
    let steps = json!([
        { "uuid": "if", "action": "control.if",
          "params": { "input": "{{input}}", "op": "gt", "operand": 10 },
          "body": [ { "uuid": "y", "action": "scripting.output", "params": { "value": "big" } } ],
          "otherwise": [ { "uuid": "n", "action": "scripting.output", "params": { "value": "small" } } ] }
    ]);
    let host = || Arc::new(MockHost::default());
    let big = run_with(host(), Value::Number(42.0), steps.clone())
        .await
        .unwrap();
    let small = run_with(host(), Value::text("3"), steps).await.unwrap();
    assert_eq!(big, Value::text("big"));
    assert_eq!(small, Value::text("small"));
}

#[tokio::test]
async fn repeat_collects_each_turn() {
    let out = run(json!([
        { "uuid": "r", "action": "control.repeat", "params": { "count": 3 },
          "body": [ { "uuid": "c", "action": "scripting.comment", "params": { "text": "" } } ] }
    ]))
    .await
    .unwrap();
    assert_eq!(out, Value::List(vec![1.0.into(), 2.0.into(), 3.0.into()]));
}

#[tokio::test]
async fn repeat_each_exposes_item_and_index() {
    let host = Arc::new(MockHost::default());
    run_with(
        host.clone(),
        Value::Nothing,
        json!([
            { "uuid": "r", "action": "control.repeat_each", "params": { "items": ["a", "b"] },
              "body": [ { "uuid": "s", "action": "scripting.show_result",
                          "params": { "content": "{{index}}={{item}}" } } ] }
        ]),
    )
    .await
    .unwrap();
    assert_eq!(
        *host.shown.lock().unwrap(),
        vec![Value::text("1=a"), Value::text("2=b")]
    );
}

#[tokio::test]
async fn nested_loops_see_their_own_item() {
    let host = Arc::new(MockHost::default());
    run_with(
        host.clone(),
        Value::Nothing,
        json!([
            { "uuid": "outer", "action": "control.repeat_each", "params": { "items": ["x", "y"] },
              "body": [
                { "uuid": "inner", "action": "control.repeat", "params": { "count": 2 },
                  "body": [ { "uuid": "s", "action": "scripting.show_result",
                              "params": { "content": "{{index}}" } } ] },
                { "uuid": "t", "action": "scripting.show_result", "params": { "content": "{{item}}" } }
              ] }
        ]),
    )
    .await
    .unwrap();
    let shown: Vec<String> = host
        .shown
        .lock()
        .unwrap()
        .iter()
        .map(Value::to_text)
        .collect();
    assert_eq!(shown, ["1", "2", "x", "1", "2", "y"]);
}

#[tokio::test]
async fn menu_runs_chosen_case() {
    let steps = json!([
        { "uuid": "m", "action": "control.menu", "params": { "prompt": "Mode?" },
          "cases": [
            { "label": "Work", "body": [ { "uuid": "w", "action": "scripting.output", "params": { "value": "work" } } ] },
            { "label": "Play", "body": [ { "uuid": "p", "action": "scripting.output", "params": { "value": "play" } } ] }
          ] }
    ]);
    let picked = run_with(
        Arc::new(MockHost::answering([Answer::Pick(1)])),
        Value::Nothing,
        steps.clone(),
    );
    assert_eq!(picked.await.unwrap(), Value::text("play"));

    let dismissed = run_with(Arc::new(MockHost::default()), Value::Nothing, steps);
    assert_eq!(dismissed.await.unwrap(), Value::Nothing);
}

#[tokio::test]
async fn dismissing_a_prompt_stops_quietly() {
    let host = Arc::new(MockHost::answering([Answer::Dismiss]));
    let out = run_with(
        host.clone(),
        Value::Nothing,
        json!([
            { "uuid": "a", "action": "scripting.ask_input", "params": { "prompt": "Name?" } },
            { "uuid": "b", "action": "scripting.show_result" },
        ]),
    )
    .await;
    assert_eq!(out, Ok(Value::Nothing));
    assert!(host.shown.lock().unwrap().is_empty());
}

#[tokio::test]
async fn ask_input_answer_feeds_next_step() {
    let host = Arc::new(MockHost::answering([Answer::Text("Ada")]));
    let out = run_with(
        host,
        Value::Nothing,
        json!([
            { "uuid": "a", "action": "scripting.ask_input", "params": { "prompt": "Name?" } },
            { "uuid": "b", "action": "scripting.output", "params": { "value": "hi {{output:a}}" } },
        ]),
    )
    .await;
    assert_eq!(out, Ok(Value::text("hi Ada")));
}

#[tokio::test]
async fn errors_point_at_the_failing_step() {
    let err = run(json!([
        { "uuid": "ok", "action": "scripting.nothing" },
        { "uuid": "bad", "action": "does.not_exist" },
    ]))
    .await
    .unwrap_err();
    assert_eq!(err.step.as_deref(), Some("bad"));
    assert_eq!(err.message, "unknown action 'does.not_exist'");
}

#[tokio::test]
async fn invalid_params_are_reported() {
    let err =
        run(json!([{ "uuid": "w", "action": "scripting.wait", "params": { "seconds": "soon" } }]))
            .await
            .unwrap_err();
    assert_eq!(
        err.message,
        "parameter 'seconds': cannot use text as number"
    );

    let err =
        run(json!([{ "uuid": "c", "action": "scripting.count", "params": { "unit": "bytes" } }]))
            .await
            .unwrap_err();
    assert!(
        err.message.contains("not one of the choices"),
        "{}",
        err.message
    );

    let err = run(json!([{ "uuid": "r", "action": "scripting.run_shortcut" }]))
        .await
        .unwrap_err();
    assert_eq!(err.message, "missing parameter 'name'");
}

#[tokio::test]
async fn disabled_steps_are_skipped() {
    let out = run(json!([
        { "uuid": "a", "action": "scripting.output", "params": { "value": "no" }, "disabled": true },
        { "uuid": "b", "action": "scripting.output", "params": { "value": "yes" } },
    ]))
    .await;
    assert_eq!(out, Ok(Value::text("yes")));
}

#[tokio::test]
async fn events_follow_step_order() {
    let host = Arc::new(MockHost::default());
    run_with(
        host.clone(),
        Value::Nothing,
        json!([
            { "uuid": "a", "action": "scripting.nothing" },
            { "uuid": "if", "action": "control.if", "params": { "input": "x" },
              "body": [ { "uuid": "b", "action": "scripting.nothing" } ] },
        ]),
    )
    .await
    .unwrap();
    assert_eq!(host.started_steps(), ["a", "if", "b"]);
}

#[tokio::test]
async fn run_shortcut_isolates_variables_and_returns_output() {
    let host = MockHost::default().with_shortcut(
        "Greet",
        json!([{ "uuid": "g", "action": "scripting.output", "params": { "value": "hello {{input}}" } }]),
    );
    let out = run_with(
        Arc::new(host),
        Value::Nothing,
        json!([{ "uuid": "r", "action": "scripting.run_shortcut",
                 "params": { "name": "Greet", "input": "Ada" } }]),
    )
    .await;
    assert_eq!(out, Ok(Value::text("hello Ada")));
}

#[tokio::test]
async fn recursive_shortcuts_hit_the_depth_limit() {
    let host = MockHost::default().with_shortcut(
        "Loop",
        json!([{ "uuid": "r", "action": "scripting.run_shortcut", "params": { "name": "Loop" } }]),
    );
    let err = run_with(
        Arc::new(host),
        Value::Nothing,
        json!([{ "uuid": "r", "action": "scripting.run_shortcut", "params": { "name": "Loop" } }]),
    )
    .await
    .unwrap_err();
    assert!(err.message.contains("levels deep"), "{}", err.message);
}

struct Slow;

#[async_trait]
impl Action for Slow {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new(
            "test.slow",
            Category::Scripting,
            "turtle",
            l("Slow", "Lent"),
        )
    }

    async fn run(&self, _: &mut RunContext, _: Value, _: Params) -> Result<Value, ActionError> {
        tokio::time::sleep(Duration::from_secs(3600)).await;
        Ok(Value::Nothing)
    }
}

fn engine_with_slow(timeout: Duration) -> Engine {
    let mut registry = Registry::builtin();
    registry.add(Arc::new(Slow));
    Engine::new(
        registry,
        EngineConfig {
            action_timeout: timeout,
        },
    )
}

#[tokio::test(start_paused = true)]
async fn bounded_actions_time_out() {
    let engine = engine_with_slow(Duration::from_secs(5));
    let wf = workflow_from(json!([{ "uuid": "s", "action": "test.slow" }]));
    let err = engine
        .run(
            &wf,
            Value::Nothing,
            Arc::new(MockHost::default()),
            CancellationToken::new(),
        )
        .await
        .unwrap_err();
    assert_eq!(err.message, "took longer than 5s");
}

#[tokio::test(start_paused = true)]
async fn cancel_interrupts_a_wait() {
    let engine = engine_with_slow(Duration::from_secs(5));
    let wf = workflow_from(json!([
        { "uuid": "w", "action": "scripting.wait", "params": { "seconds": 600 } },
        { "uuid": "after", "action": "scripting.nothing" },
    ]));
    let cancel = CancellationToken::new();
    let trigger = cancel.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        trigger.cancel();
    });
    let err = engine
        .run(&wf, Value::Nothing, Arc::new(MockHost::default()), cancel)
        .await
        .unwrap_err();
    assert!(err.cancelled);
    assert_eq!(err.step.as_deref(), Some("w"));
}
