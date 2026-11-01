use std::collections::HashMap;
use std::sync::Arc;

use super::action::{Action, ActionMeta};

#[doc(hidden)]
pub struct Registration(pub fn() -> Arc<dyn Action>);

inventory::collect!(Registration);

/// Registers a built-in action so the registry picks it up at startup.
///
/// ```ignore
/// pub struct Beep;
/// impl Action for Beep { ... }
/// register_action!(Beep);
/// ```
#[macro_export]
macro_rules! register_action {
    ($action:expr) => {
        inventory::submit! {
            $crate::engine::Registration(|| std::sync::Arc::new($action))
        }
    };
}

#[derive(Clone)]
pub struct Entry {
    pub action: Arc<dyn Action>,
    pub meta: ActionMeta,
}

#[derive(Clone, Default)]
pub struct Registry {
    entries: HashMap<String, Entry>,
}

impl Registry {
    pub fn builtin() -> Self {
        let mut registry = Registry::default();
        for reg in inventory::iter::<Registration> {
            registry.add((reg.0)());
        }
        registry
    }

    pub fn add(&mut self, action: Arc<dyn Action>) {
        let meta = action.meta();
        debug_assert!(
            !self.entries.contains_key(&meta.id),
            "duplicate action id {}",
            meta.id
        );
        self.entries.insert(meta.id.clone(), Entry { action, meta });
    }

    pub fn get(&self, id: &str) -> Option<&Entry> {
        self.entries.get(id)
    }

    pub fn metas(&self) -> Vec<ActionMeta> {
        let mut metas: Vec<_> = self.entries.values().map(|e| e.meta.clone()).collect();
        metas.sort_by(|a, b| a.id.cmp(&b.id));
        metas
    }
}
