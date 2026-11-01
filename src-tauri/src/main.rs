// hides the extra console window on release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    shortcuteo_lib::run()
}
