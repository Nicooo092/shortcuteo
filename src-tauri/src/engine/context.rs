use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use super::action::{BlockKind, ParamKind, ParamSpec};
use super::error::{ActionError, RunError};
use super::host::{Host, RunEvent};
use super::params::Params;
use super::registry::Entry;
use super::template;
use super::value::Value;
use super::Engine;
use crate::storage::workflow::Step;

const MAX_DEPTH: u32 = 16;
const MAX_REPEAT: f64 = 100_000.0;

pub(super) enum Flow {
    Stop(Value),
    Fail(RunError),
}

type StepsFuture<'a> = Pin<Box<dyn Future<Output = Result<Value, Flow>> + Send + 'a>>;

struct Loop {
    item: Value,
    index: usize,
}

/// State of one run, handed to every action.
pub struct RunContext {
    pub host: Arc<dyn Host>,
    pub cancel: CancellationToken,
    engine: Engine,
    input: Value,
    variables: HashMap<String, Value>,
    outputs: HashMap<String, Value>,
    loops: Vec<Loop>,
    depth: u32,
}

impl RunContext {
    pub(super) fn new(
        engine: Engine,
        host: Arc<dyn Host>,
        cancel: CancellationToken,
        input: Value,
        depth: u32,
    ) -> Self {
        RunContext {
            host,
            cancel,
            engine,
            input,
            variables: HashMap::new(),
            outputs: HashMap::new(),
            loops: Vec::new(),
            depth,
        }
    }

    pub fn shortcut_input(&self) -> &Value {
        &self.input
    }

    pub fn variable(&self, name: &str) -> Value {
        self.variables.get(name).cloned().unwrap_or(Value::Nothing)
    }

    pub fn set_variable(&mut self, name: &str, value: Value) {
        self.variables.insert(name.to_string(), value);
    }

    /// Runs another saved shortcut with its own variables, sharing host and cancellation.
    pub async fn run_shortcut(&mut self, name: &str, input: Value) -> Result<Value, ActionError> {
        if self.depth >= MAX_DEPTH {
            return Err(ActionError::failed(format!(
                "shortcuts call each other more than {MAX_DEPTH} levels deep"
            )));
        }
        let workflow = self
            .host
            .find_shortcut(name)
            .ok_or_else(|| ActionError::failed(format!("no shortcut named '{name}'")))?;
        let mut child = RunContext::new(
            self.engine.clone(),
            self.host.clone(),
            self.cancel.clone(),
            input.clone(),
            self.depth + 1,
        );
        match child.run_steps(&workflow.actions, input).await {
            Ok(v) | Err(Flow::Stop(v)) => Ok(v),
            Err(Flow::Fail(e)) if e.cancelled => Err(ActionError::Cancelled),
            Err(Flow::Fail(e)) => Err(ActionError::failed(format!("in '{name}': {e}"))),
        }
    }

    pub(super) fn token(&self, token: &str) -> Option<Value> {
        let value = match token.split_once(':') {
            Some(("output", uuid)) => self.outputs.get(uuid).cloned().unwrap_or(Value::Nothing),
            Some(("var", name)) => self.variable(name),
            Some(_) => return None,
            None => match token {
                "input" => self.input.clone(),
                "item" => self
                    .loops
                    .last()
                    .map(|l| l.item.clone())
                    .unwrap_or(Value::Nothing),
                "index" => self
                    .loops
                    .last()
                    .map(|l| Value::Number(l.index as f64))
                    .unwrap_or(Value::Nothing),
                "now" => Value::Date(chrono::Local::now().fixed_offset()),
                _ => return None,
            },
        };
        Some(value)
    }

    pub(super) fn run_steps<'a>(&'a mut self, steps: &'a [Step], input: Value) -> StepsFuture<'a> {
        Box::pin(async move {
            let mut last = input;
            for step in steps.iter().filter(|s| !s.disabled) {
                last = self.run_step(step, last).await?;
            }
            Ok(last)
        })
    }

    async fn run_step(&mut self, step: &Step, input: Value) -> Result<Value, Flow> {
        let host = self.host.clone();
        let fail = |e: &ActionError| {
            host.on_event(RunEvent::StepFailed {
                step: step.uuid.clone(),
                message: e.to_string(),
            });
            match e {
                ActionError::Stop(v) => Flow::Stop(v.clone()),
                _ => Flow::Fail(RunError::at(&step.uuid, &step.action, e)),
            }
        };

        if self.cancel.is_cancelled() {
            return Err(fail(&ActionError::Cancelled));
        }
        let entry = self
            .engine
            .registry()
            .get(&step.action)
            .cloned()
            .ok_or_else(|| fail(&ActionError::UnknownAction(step.action.clone())))?;
        host.on_event(RunEvent::StepStarted {
            step: step.uuid.clone(),
        });

        let params = self
            .resolve_params(&entry.meta.params, step, &input)
            .map_err(|e| fail(&e))?;
        let output = match entry.meta.block {
            None => self
                .invoke(&entry, input, params)
                .await
                .map_err(|e| fail(&e))?,
            Some(BlockKind::If) => {
                let cond = self
                    .invoke(&entry, input.clone(), params)
                    .await
                    .map_err(|e| fail(&e))?;
                let branch = if cond.to_bool() {
                    &step.body
                } else {
                    &step.otherwise
                };
                self.run_steps(branch, input).await?
            }
            Some(BlockKind::Repeat) => {
                let count = self
                    .invoke(&entry, input, params)
                    .await
                    .map_err(|e| fail(&e))?;
                let count = count
                    .to_number()
                    .map_err(|e| fail(&ActionError::failed(e.to_string())))?
                    .clamp(0.0, MAX_REPEAT) as usize;
                let items = (1..=count).map(|i| Value::Number(i as f64)).collect();
                self.run_loop(&step.body, items).await?
            }
            Some(BlockKind::RepeatEach) => {
                let items = self
                    .invoke(&entry, input, params)
                    .await
                    .map_err(|e| fail(&e))?
                    .into_list();
                self.run_loop(&step.body, items).await?
            }
            Some(BlockKind::Menu) => {
                let labels: Vec<String> = step
                    .cases
                    .iter()
                    .map(|c| template::render(&c.label, self).to_text())
                    .collect();
                let picked = host
                    .choose(&params.text("prompt"), &labels, false)
                    .await
                    .map_err(|e| fail(&e))?;
                match picked.as_deref() {
                    Some([i, ..]) if *i < step.cases.len() => {
                        self.run_steps(&step.cases[*i].body, input).await?
                    }
                    _ => return Err(fail(&ActionError::Stop(Value::Nothing))),
                }
            }
        };

        self.outputs.insert(step.uuid.clone(), output.clone());
        host.on_event(RunEvent::StepFinished {
            step: step.uuid.clone(),
            output: output.clone(),
        });
        Ok(output)
    }

    async fn invoke(
        &mut self,
        entry: &Entry,
        input: Value,
        params: Params,
    ) -> Result<Value, ActionError> {
        let cancel = self.cancel.clone();
        let timeout = (!entry.meta.unbounded).then(|| self.engine.action_timeout());
        let run = entry.action.run(self, input, params);
        let bounded = async {
            match timeout {
                Some(t) => tokio::time::timeout(t, run)
                    .await
                    .unwrap_or(Err(ActionError::Timeout(t))),
                None => run.await,
            }
        };
        tokio::select! {
            biased;
            _ = cancel.cancelled() => Err(ActionError::Cancelled),
            r = bounded => r,
        }
    }

    async fn run_loop(&mut self, body: &[Step], items: Vec<Value>) -> Result<Value, Flow> {
        let mut results = Vec::with_capacity(items.len());
        for (i, item) in items.into_iter().enumerate() {
            self.loops.push(Loop {
                item: item.clone(),
                index: i + 1,
            });
            let r = self.run_steps(body, item).await;
            self.loops.pop();
            results.push(r?);
        }
        Ok(Value::List(results))
    }

    fn resolve_params(
        &self,
        specs: &[ParamSpec],
        step: &Step,
        input: &Value,
    ) -> Result<Params, ActionError> {
        let mut params = Params::new();
        for spec in specs {
            let mut value = step
                .params
                .get(&spec.key)
                .map(|j| self.resolve_json(j))
                .unwrap_or(Value::Nothing);
            if spec.from_input && is_blank(&value) {
                value = input.clone();
            }
            if value.is_nothing() {
                if let Some(default) = &spec.default {
                    value = self.resolve_json(default);
                }
            }
            if value.is_nothing() {
                if spec.optional {
                    continue;
                }
                return Err(ActionError::MissingParam(spec.key.clone()));
            }
            let value =
                value
                    .coerce(spec.kind.value_type())
                    .map_err(|source| ActionError::BadParam {
                        param: spec.key.clone(),
                        source,
                    })?;
            if let ParamKind::Choice { options } = &spec.kind {
                let picked = value.to_text();
                if !options.iter().any(|o| o.value == picked) {
                    return Err(ActionError::failed(format!(
                        "parameter '{}': '{picked}' is not one of the choices",
                        spec.key
                    )));
                }
            }
            params.insert(spec.key.clone(), value);
        }
        Ok(params)
    }

    fn resolve_json(&self, json: &serde_json::Value) -> Value {
        match json {
            serde_json::Value::String(s) => template::render(s, self),
            serde_json::Value::Array(items) => {
                Value::List(items.iter().map(|j| self.resolve_json(j)).collect())
            }
            serde_json::Value::Object(map) => Value::Dict(
                map.iter()
                    .map(|(k, v)| (k.clone(), self.resolve_json(v)))
                    .collect(),
            ),
            other => Value::from_json(other.clone()),
        }
    }
}

fn is_blank(v: &Value) -> bool {
    match v {
        Value::Nothing => true,
        Value::Text(s) => s.is_empty(),
        _ => false,
    }
}
