use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::context::RunContext;
use super::error::ActionError;
use super::params::Params;
use super::value::{Value, ValueType};

/// A single block of a shortcut.
///
/// Implement this, describe the block in [`ActionMeta`], then register it with
/// [`register_action!`](crate::register_action). The front builds the block, its parameter
/// form and its picker entry from the metadata alone.
#[async_trait]
pub trait Action: Send + Sync {
    fn meta(&self) -> ActionMeta;

    /// `input` is the output of the previous step. `params` are already resolved and coerced
    /// to the types declared in [`ActionMeta::params`].
    async fn run(
        &self,
        ctx: &mut RunContext,
        input: Value,
        params: Params,
    ) -> Result<Value, ActionError>;
}

/// Text shown to the user, in every supported language.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct Localized {
    pub en: String,
    pub fr: String,
}

pub fn l(en: &str, fr: &str) -> Localized {
    Localized {
        en: en.to_string(),
        fr: fr.to_string(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Category {
    Scripting,
    ControlFlow,
    Variables,
    Lists,
    Math,
    Dates,
    Text,
    Audio,
    Files,
    Web,
    System,
    Display,
    Navigation,
    Capture,
    Device,
    Clipboard,
    Wallpaper,
    Windows,
}

/// Category tile colors, see the design tokens on the front.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Tile {
    Iris,
    Ember,
    Lagoon,
    Amber,
    Orchid,
    Glacier,
    Slate,
}

impl Category {
    pub fn tile(self) -> Tile {
        use Category::*;
        match self {
            Scripting | ControlFlow | Windows => Tile::Slate,
            Variables | Lists => Tile::Orchid,
            Math | Dates => Tile::Amber,
            Text | Clipboard => Tile::Iris,
            Audio | Capture => Tile::Ember,
            Files | Web => Tile::Glacier,
            System | Display | Navigation | Device | Wallpaper => Tile::Lagoon,
        }
    }
}

/// Blocks that hold other steps. The interpreter drives them, the action only decides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum BlockKind {
    /// `run` returns a bool: `body` runs on true, `otherwise` on false.
    If,
    /// `run` returns a count: `body` runs that many times.
    Repeat,
    /// `run` returns a list: `body` runs once per item.
    RepeatEach,
    /// `run` is not called: the user picks one of the step's `cases`.
    Menu,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct ChoiceOption {
    pub value: String,
    pub label: Localized,
}

pub fn choice(value: &str, en: &str, fr: &str) -> ChoiceOption {
    ChoiceOption {
        value: value.to_string(),
        label: l(en, fr),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ParamKind {
    Text {
        multiline: bool,
    },
    Number {
        min: Option<f64>,
        max: Option<f64>,
        unit: Option<String>,
    },
    Bool,
    Choice {
        options: Vec<ChoiceOption>,
    },
    /// name of a shortcut variable, shown as a variable pill
    Variable,
    Any,
}

impl ParamKind {
    pub fn value_type(&self) -> ValueType {
        match self {
            ParamKind::Text { .. } | ParamKind::Choice { .. } | ParamKind::Variable => {
                ValueType::Text
            }
            ParamKind::Number { .. } => ValueType::Number,
            ParamKind::Bool => ValueType::Bool,
            ParamKind::Any => ValueType::Any,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ParamSpec {
    pub key: String,
    pub label: Localized,
    pub kind: ParamKind,
    #[ts(type = "unknown")]
    pub default: Option<serde_json::Value>,
    pub optional: bool,
    /// filled with the previous step's output when left empty
    pub from_input: bool,
}

impl ParamSpec {
    pub fn new(key: &str, en: &str, fr: &str, kind: ParamKind) -> Self {
        ParamSpec {
            key: key.to_string(),
            label: l(en, fr),
            kind,
            default: None,
            optional: false,
            from_input: false,
        }
    }

    pub fn text(key: &str, en: &str, fr: &str) -> Self {
        Self::new(key, en, fr, ParamKind::Text { multiline: false })
    }

    pub fn multiline(key: &str, en: &str, fr: &str) -> Self {
        Self::new(key, en, fr, ParamKind::Text { multiline: true })
    }

    pub fn number(key: &str, en: &str, fr: &str) -> Self {
        Self::new(
            key,
            en,
            fr,
            ParamKind::Number {
                min: None,
                max: None,
                unit: None,
            },
        )
    }

    pub fn boolean(key: &str, en: &str, fr: &str) -> Self {
        Self::new(key, en, fr, ParamKind::Bool)
    }

    pub fn choice(key: &str, en: &str, fr: &str, options: Vec<ChoiceOption>) -> Self {
        Self::new(key, en, fr, ParamKind::Choice { options })
    }

    pub fn any(key: &str, en: &str, fr: &str) -> Self {
        Self::new(key, en, fr, ParamKind::Any)
    }

    pub fn variable(key: &str, en: &str, fr: &str) -> Self {
        Self::new(key, en, fr, ParamKind::Variable)
    }

    pub fn default(mut self, value: impl Into<serde_json::Value>) -> Self {
        self.default = Some(value.into());
        self
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn from_input(mut self) -> Self {
        self.from_input = true;
        self
    }

    pub fn range(mut self, lo: f64, hi: f64) -> Self {
        if let ParamKind::Number { min, max, .. } = &mut self.kind {
            *min = Some(lo);
            *max = Some(hi);
        }
        self
    }

    pub fn unit(mut self, u: &str) -> Self {
        if let ParamKind::Number { unit, .. } = &mut self.kind {
            *unit = Some(u.to_string());
        }
        self
    }
}

/// Everything the front needs to show an action.
#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ActionMeta {
    /// stable id, `category.name`; renaming it breaks saved shortcuts
    pub id: String,
    pub category: Category,
    pub tile: Tile,
    /// lucide icon name
    pub icon: String,
    pub name: Localized,
    pub description: Localized,
    /// block phrase, `{key}` placeholders become inline parameter pills
    pub sentence: Localized,
    pub params: Vec<ParamSpec>,
    pub input: ValueType,
    pub output: ValueType,
    pub block: Option<BlockKind>,
    /// asks for confirmation the first time an imported shortcut runs it
    pub sensitive: bool,
    /// waits on the user or on purpose, so the per-action timeout does not apply
    pub unbounded: bool,
    pub experimental: bool,
}

impl ActionMeta {
    pub fn new(id: &str, category: Category, icon: &str, name: Localized) -> Self {
        ActionMeta {
            id: id.to_string(),
            category,
            tile: category.tile(),
            icon: icon.to_string(),
            sentence: name.clone(),
            name,
            description: l("", ""),
            params: Vec::new(),
            input: ValueType::Nothing,
            output: ValueType::Nothing,
            block: None,
            sensitive: false,
            unbounded: false,
            experimental: false,
        }
    }

    pub fn describe(mut self, en: &str, fr: &str) -> Self {
        self.description = l(en, fr);
        self
    }

    pub fn sentence(mut self, en: &str, fr: &str) -> Self {
        self.sentence = l(en, fr);
        self
    }

    pub fn param(mut self, p: ParamSpec) -> Self {
        self.params.push(p);
        self
    }

    pub fn input(mut self, t: ValueType) -> Self {
        self.input = t;
        self
    }

    pub fn output(mut self, t: ValueType) -> Self {
        self.output = t;
        self
    }

    pub fn block(mut self, b: BlockKind) -> Self {
        self.block = Some(b);
        self
    }

    pub fn sensitive(mut self) -> Self {
        self.sensitive = true;
        self
    }

    pub fn unbounded(mut self) -> Self {
        self.unbounded = true;
        self
    }
}
