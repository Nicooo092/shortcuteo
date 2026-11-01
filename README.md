# Shortcuteo

Action shortcuts for Windows. Chain small blocks (set volume, copy text, run PowerShell, open an app...) into a shortcut, then run it from a hotkey, the tray, a schedule or the command line.

Heavily inspired by the Shortcuts app on iOS, built with Tauri 2 and React.

![Shortcuteo](docs/screenshot.svg)

Early work in progress. Nothing is usable yet, see the [changelog](CHANGELOG.md).

## Install

Installers (MSI and NSIS) will be attached to each [release](../../releases) once the first one ships.

To build from source you need Rust (stable), Node 22, the MSVC build tools and WebView2 (preinstalled on Windows 11):

```sh
npm install
npm run tauri dev     # run in dev mode
npm run tauri build   # produce installers in src-tauri/target/release/bundle
```

## Example shortcut

A shortcut is a `.shortcuteo` JSON file. Drop it on the window to import it.

```json
{
  "schemaVersion": 1,
  "name": "Focus",
  "actions": [
    { "id": "system.set_volume", "params": { "level": 20 } },
    { "id": "scripting.wait", "params": { "seconds": 1500 } },
    { "id": "notification.show", "params": { "title": "Break time" } }
  ]
}
```

The format is not frozen yet and may change before 0.1.

## Contributing

Adding an action is one Rust file. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE)
