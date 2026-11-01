# Contributing

Bug reports, ideas and pull requests are welcome. For anything bigger than a fix, open an issue first so we can agree on the shape.

## Setup

```sh
npm install
npm run tauri dev
```

Before pushing:

```sh
npm run lint && npm run typecheck && npm run format:check
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
```

CI runs the same checks on `windows-latest`.

## Layout

- `src-tauri/src/engine` runs workflows: values, variables, control flow.
- `src-tauri/src/actions` holds one file per action, grouped by category.
- `src-tauri/src/triggers` starts shortcuts (hotkey, schedule, CLI...).
- `src/` is the React front. It builds blocks and forms from action metadata, so most actions need no UI code.

## Add an action in 5 minutes

This section will be filled in when the `Action` trait lands (milestone M1).

## Style

- Comments are short, in English, and explain why, not what.
- Commit messages are short and imperative: `add volume action`, `fix hotkey after sleep`.
- One logical change per commit.
