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
        ])
        .run(tauri::generate_context!())
        .expect("failed to start tauri runtime");
}
