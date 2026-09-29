#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_repository_info,
            commands::get_hosts,
            commands::get_options,
            commands::inspect_option,
            commands::plan_mutation,
            commands::apply_transaction,
        ])
        .run(tauri::generate_context!())
        .expect("error while running avalanche desktop");
}
