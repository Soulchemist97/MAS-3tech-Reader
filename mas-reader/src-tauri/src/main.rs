// Unter Windows kein Konsolenfenster im Release-Build
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::AppState::default())
        .invoke_handler(commands::handler())
        .run(commands::context())
        .expect("MAS Reader konnte nicht gestartet werden");
}
