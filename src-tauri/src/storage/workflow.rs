use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::engine::Tile;

pub const SCHEMA_VERSION: u32 = 1;

/// A shortcut, as stored in a `.shortcuteo` file.
///
/// Parameter values are plain JSON. Inside strings, `{{...}}` tokens pull in magic variables:
///
/// - `{{output:<step uuid>}}` the output of an earlier step
/// - `{{var:<name>}}` a variable set with "Set variable"
/// - `{{input}}` the shortcut input
/// - `{{item}}` / `{{index}}` the current element and 1-based index inside a repeat
/// - `{{now}}` the current date
///
/// A string made of a single token keeps the referenced value's type; anything else becomes
/// text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Workflow {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default = "default_tile")]
    pub color: Tile,
    #[serde(default)]
    pub favorite: bool,
    /// library folder name, purely for organizing
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub folder: Option<String>,
    #[serde(default)]
    pub actions: Vec<Step>,
}

/// One block. Container blocks keep their children in `body`, `otherwise` and `cases`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Step {
    /// unique inside the workflow, referenced by `{{output:...}}`
    pub uuid: String,
    /// action id from the registry, e.g. `text.change_case`
    pub action: String,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    #[ts(type = "Record<string, unknown>")]
    pub params: IndexMap<String, serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub body: Vec<Step>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub otherwise: Vec<Step>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cases: Vec<MenuCase>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct MenuCase {
    pub label: String,
    #[serde(default)]
    pub body: Vec<Step>,
}

fn default_icon() -> String {
    "sparkles".to_string()
}

fn default_tile() -> Tile {
    Tile::Iris
}

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("not a shortcut file: {0}")]
    Json(#[from] serde_json::Error),
    #[error("missing schemaVersion")]
    NoVersion,
    #[error(
        "made with a newer Shortcuteo (format v{0}, this build reads up to v{SCHEMA_VERSION})"
    )]
    TooNew(u32),
}

type Migration = fn(&mut serde_json::Value);

// MIGRATIONS[n] upgrades a document from version n+1 to n+2. append only, never edit one
// that shipped: old files in the wild depend on it.
const MIGRATIONS: &[Migration] = &[];

pub fn parse(text: &str) -> Result<Workflow, FormatError> {
    let mut doc: serde_json::Value = serde_json::from_str(text)?;
    migrate(&mut doc, MIGRATIONS)?;
    Ok(serde_json::from_value(doc)?)
}

pub fn to_string(workflow: &Workflow) -> String {
    serde_json::to_string_pretty(workflow).expect("workflow is always serializable")
}

fn migrate(doc: &mut serde_json::Value, migrations: &[Migration]) -> Result<(), FormatError> {
    let latest = migrations.len() as u32 + 1;
    let version = doc
        .get("schemaVersion")
        .and_then(serde_json::Value::as_u64)
        .ok_or(FormatError::NoVersion)? as u32;
    if version > latest {
        return Err(FormatError::TooNew(version));
    }
    for m in &migrations[(version.max(1) - 1) as usize..] {
        m(doc);
    }
    doc["schemaVersion"] = latest.into();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "schemaVersion": 1,
        "id": "focus",
        "name": "Focus",
        "actions": [
            { "uuid": "a", "action": "control.if",
              "params": { "input": "{{input}}", "op": "has_value" },
              "body": [ { "uuid": "b", "action": "text.text", "params": { "text": "yes" } } ],
              "otherwise": [ { "uuid": "c", "action": "scripting.nothing" } ] }
        ]
    }"#;

    #[test]
    fn parses_nested_steps_and_defaults() {
        let wf = parse(SAMPLE).unwrap();
        assert_eq!(wf.icon, "sparkles");
        assert_eq!(wf.actions[0].body[0].params["text"], "yes");
        assert_eq!(wf.actions[0].otherwise[0].action, "scripting.nothing");
    }

    #[test]
    fn roundtrip_is_stable() {
        let wf = parse(SAMPLE).unwrap();
        assert_eq!(parse(&to_string(&wf)).unwrap(), wf);
    }

    #[test]
    fn rejects_newer_and_unversioned_files() {
        assert!(matches!(
            parse(r#"{"schemaVersion": 99}"#),
            Err(FormatError::TooNew(99))
        ));
        assert!(matches!(
            parse(r#"{"name": "x"}"#),
            Err(FormatError::NoVersion)
        ));
    }

    #[test]
    fn migrations_run_in_order_from_file_version() {
        fn v1_to_v2(doc: &mut serde_json::Value) {
            doc["log"] = serde_json::json!(["2"]);
        }
        fn v2_to_v3(doc: &mut serde_json::Value) {
            doc["log"].as_array_mut().unwrap().push("3".into());
        }
        let chain: &[Migration] = &[v1_to_v2, v2_to_v3];

        let mut from_v1 = serde_json::json!({ "schemaVersion": 1 });
        migrate(&mut from_v1, chain).unwrap();
        assert_eq!(from_v1["log"], serde_json::json!(["2", "3"]));
        assert_eq!(from_v1["schemaVersion"], 3);

        let mut from_v2 = serde_json::json!({ "schemaVersion": 2, "log": [] });
        migrate(&mut from_v2, chain).unwrap();
        assert_eq!(from_v2["log"], serde_json::json!(["3"]));
    }
}
