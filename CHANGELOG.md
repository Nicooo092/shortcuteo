# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [semantic versioning](https://semver.org/).

## [Unreleased]

### Added

- Tauri 2 and React scaffold, lint and format config, CI on Windows.
- Workflow engine: typed values with implicit conversions, magic variables, If / Repeat / Repeat with each / Menu, nested shortcuts, per-action timeout and cancellation.
- `.shortcuteo` file format with schema versioning and migrations.
- 31 actions: scripting and control flow, variables, text, clipboard, notifications, open app or URL, PowerShell and cmd.
- IPC commands to list actions, run and cancel shortcuts, and manage the saved library.
- Glass interface: Mica window, custom title bar, drifting ambient background, refraction on search fields and notifications.
- Library with favorites, folders and fuzzy search; click runs a shortcut, double-click edits it.
- Block editor: inline parameter pills, magic variables, nested If / Repeat / Menu blocks, drag and drop, inspector, step-by-step run highlighting.
- Action sheet with category filter and keyboard navigation, `Ctrl+K` palette.
- Gallery of example shortcuts, drop a `.shortcuteo` file on the window to import it.
- French and English interface, accent color, blur intensity and animation settings.
