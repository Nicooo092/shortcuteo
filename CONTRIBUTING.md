# Contributing

Bug reports, ideas and pull requests are welcome. For anything bigger than a fix, open an issue first so we can agree on the shape.

## Setup

```sh
npm install
npm run tauri dev
```

Working only on the interface? `npm run dev` opens it in a normal browser at http://localhost:1420 with a fake engine and the real action catalog (`src/bindings/catalog.json`). Dialogs, step highlighting and the editor all work; actions themselves do not run.

Before pushing:

```sh
npm run lint && npm run typecheck && npm run format:check && npm test
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

CI runs the same checks on `windows-latest`.

## Layout

- `src-tauri/src/engine` runs workflows: values, variables, control flow.
- `src-tauri/src/actions` holds one file per action, grouped by category.
- `src-tauri/src/triggers` starts shortcuts (hotkey, schedule, CLI...).
- `src/` is the React front. It builds blocks and forms from action metadata, so most actions need no UI code.

## Add an action in 5 minutes

An action is one file in `src-tauri/src/actions/<category>/`. Say we want "Reverse text".

1. Create `src-tauri/src/actions/text/reverse.rs`:

```rust
use async_trait::async_trait;

use crate::engine::*;
use crate::register_action;

pub struct Reverse;

#[async_trait]
impl Action for Reverse {
    fn meta(&self) -> ActionMeta {
        ActionMeta::new("text.reverse", Category::Text, "arrow-left-right", l("Reverse text", "Inverser le texte"))
            .describe("Writes the text backwards.", "Écrit le texte à l'envers.")
            .sentence("Reverse {text}", "Inverser {text}")
            .param(ParamSpec::text("text", "Text", "Texte").from_input().optional())
            .input(ValueType::Text)
            .output(ValueType::Text)
    }

    async fn run(&self, _ctx: &mut RunContext, _input: Value, params: Params) -> Result<Value, ActionError> {
        Ok(Value::Text(params.text("text").chars().rev().collect()))
    }
}

register_action!(Reverse);
```

2. Add `mod reverse;` to `src-tauri/src/actions/text/mod.rs`.
3. Run `cargo test`. The contract tests check that the id is unique, that every `{placeholder}` in the sentence is a parameter, and that choice defaults are valid.

That is all: the editor builds the block, its parameter pills and its entry in the action picker from `meta()`.

A few things to know:

- `id` is saved in every shortcut file. Never rename a shipped one.
- `input` is the previous step's output. A parameter marked `from_input()` is filled with it when left empty, which is how actions chain without wiring.
- Parameters arrive already converted to the declared type. Text, numbers and booleans are converted implicitly (`"3,5"` becomes `3.5`, `"non"` is false).
- Return `ActionError::failed("...")` with a message a user can act on. The engine adds the step it came from.
- Mark actions that touch the system in a way that can hurt (`sensitive()`): shell, deletion, network to a new host. Imported shortcuts ask before running them.
- Actions that wait on the user or on purpose call `unbounded()`, otherwise the per-action timeout (5 minutes) applies.
- Windows API calls go in `src-tauri/src/sys/`, so actions stay readable.
- Everything shown to users is in both English and French.

## Shortcut file format

`.shortcuteo` files are JSON, described by `Workflow` in `src-tauri/src/storage/workflow.rs`. Inside string parameters, `{{...}}` tokens insert magic variables:

| Token               | Value                                     |
| ------------------- | ----------------------------------------- |
| `{{output:<uuid>}}` | output of an earlier step                 |
| `{{var:<name>}}`    | a variable set with "Set variable"        |
| `{{input}}`         | the shortcut input                        |
| `{{item}}`          | current element inside "Repeat with each" |
| `{{index}}`         | current turn (1-based) inside a repeat    |
| `{{now}}`           | the current date                          |

A parameter that is exactly one token keeps the value's type, so a list stays a list. Any change to the format bumps `schemaVersion` and adds a migration; old files must keep opening. See `examples/` for complete shortcuts.

TypeScript types for the front are generated from the Rust types into `src/bindings/` by `cargo test`. Commit them when they change.

## Style

- Comments are short, in English, and explain why, not what.
- Commit messages are short and imperative: `add volume action`, `fix hotkey after sleep`.
- One logical change per commit.
