mod actions;
pub mod engine;
mod ipc;
pub mod storage;
mod sys;

use tauri::Manager;

use engine::{Engine, EngineConfig, Registry};
use storage::library::Library;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?.join("shortcuts");
            let engine = Engine::new(Registry::builtin(), EngineConfig::default());
            app.manage(ipc::AppState::new(engine, Library::open(dir)?));
            if let Some(window) = app.get_webview_window("main") {
                // mica needs windows 11, acrylic covers windows 10
                if window_vibrancy::apply_mica(&window, Some(true)).is_err() {
                    let _ = window_vibrancy::apply_acrylic(&window, Some((10, 10, 24, 200)));
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::list_actions,
            ipc::run_workflow,
            ipc::cancel_run,
            ipc::host_reply,
            ipc::list_shortcuts,
            ipc::save_shortcut,
            ipc::delete_shortcut,
            ipc::parse_shortcut,
            ipc::export_shortcut,
            ipc::on_battery,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start tauri runtime");
}
