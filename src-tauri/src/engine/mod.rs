mod action;
mod context;
mod error;
mod host;
mod params;
mod registry;
mod template;
mod value;

use std::sync::Arc;
use std::time::Duration;

use tokio_util::sync::CancellationToken;

pub use action::{
    choice, l, Action, ActionMeta, BlockKind, Category, ChoiceOption, Localized, ParamKind,
    ParamSpec, Tile,
};
pub use context::RunContext;
pub use error::{ActionError, RunError};
pub use host::{Host, RunEvent};
pub use params::Params;
pub use registry::{Entry, Registration, Registry};
pub use value::{ConversionError, Value, ValueType};

use crate::storage::workflow::Workflow;
use context::Flow;

#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub action_timeout: Duration,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            action_timeout: Duration::from_secs(300),
        }
    }
}

#[derive(Clone)]
pub struct Engine {
    inner: Arc<Inner>,
}

struct Inner {
    registry: Registry,
    config: EngineConfig,
}

impl Engine {
    pub fn new(registry: Registry, config: EngineConfig) -> Self {
        Engine {
            inner: Arc::new(Inner { registry, config }),
        }
    }

    pub fn registry(&self) -> &Registry {
        &self.inner.registry
    }

    pub(crate) fn action_timeout(&self) -> Duration {
        self.inner.config.action_timeout
    }

    /// Runs a workflow to completion. The result is the value given to "Stop and output",
    /// or the last step's output.
    pub async fn run(
        &self,
        workflow: &Workflow,
        input: Value,
        host: Arc<dyn Host>,
        cancel: CancellationToken,
    ) -> Result<Value, RunError> {
        let mut ctx = RunContext::new(self.clone(), host, cancel, input.clone(), 0);
        match ctx.run_steps(&workflow.actions, input).await {
            Ok(v) | Err(Flow::Stop(v)) => Ok(v),
            Err(Flow::Fail(e)) => Err(e),
        }
    }
}
