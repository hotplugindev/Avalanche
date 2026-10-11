#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::AppState;

fn main() {
    let repo_path = std::env::var("AVALANCHE_REPO").unwrap_or_else(|_| ".".to_string());
    let state = AppState::new(&repo_path);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_repository,
            commands::get_hosts,
            commands::get_users,
            commands::get_profiles,
            commands::get_applications,
            commands::get_capabilities,
            commands::get_graph,
            commands::get_doctor,
            commands::get_provenance,
            commands::search,
            commands::get_why,
            commands::get_owners,
            commands::get_dependents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running avalanche desktop");
}
