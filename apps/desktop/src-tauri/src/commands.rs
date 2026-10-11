use std::sync::Mutex;

use avalanche_core::{ConfigurationEngine, RepositoryIndex};
use avalanche_model::Repository;
use tauri::State;

pub struct AppState {
    pub repo_path: String,
    engine: Mutex<ConfigurationEngine>,
    index: Mutex<Option<RepositoryIndex>>,
}

impl AppState {
    pub fn new(repo_path: &str) -> Self {
        Self {
            repo_path: repo_path.to_string(),
            engine: Mutex::new(ConfigurationEngine::new(repo_path)),
            index: Mutex::new(None),
        }
    }
}

fn ensure_index(state: &AppState) -> Result<RepositoryIndex, String> {
    let mut idx_guard = state.index.lock().map_err(|e| e.to_string())?;
    if let Some(ref idx) = *idx_guard {
        return Ok(idx.clone());
    }
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let repo = Repository::new(&state.repo_path);
    let idx = engine
        .index_static(&repo)
        .map_err(|e| format!("failed to build index: {e}"))?;
    *idx_guard = Some(idx.clone());
    Ok(idx)
}

fn with_view<T, F>(state: &AppState, f: F) -> Result<serde_json::Value, String>
where
    F: FnOnce(&ConfigurationEngine, &RepositoryIndex) -> T,
    T: serde::Serialize,
{
    let idx = ensure_index(state)?;
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    serde_json::to_value(f(&engine, &idx)).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_repository(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    with_view(&state, |e, i| e.repository_view(i))
}

#[tauri::command]
pub fn get_hosts(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    with_view(&state, |e, i| e.hosts_view(i))
}

#[tauri::command]
pub fn get_users(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    with_view(&state, |e, i| e.users_view(i))
}

#[tauri::command]
pub fn get_profiles(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    with_view(&state, |e, i| e.profiles_view(i))
}

#[tauri::command]
pub fn get_applications(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    with_view(&state, |e, i| e.applications_view(i))
}

#[tauri::command]
pub fn get_capabilities(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    with_view(&state, |e, i| e.capabilities_view(i))
}

#[tauri::command]
pub fn get_graph(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    with_view(&state, |e, i| e.graph_view(i))
}

#[tauri::command]
pub fn get_doctor(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    with_view(&state, |e, i| e.doctor_view(i))
}

#[tauri::command]
pub fn get_provenance(
    state: State<'_, AppState>,
    option_path: String,
) -> Result<serde_json::Value, String> {
    let idx = ensure_index(&state)?;
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let prov = engine
        .provenance_view(&idx, &option_path)
        .map_err(|e| e)?;
    serde_json::to_value(prov).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search(state: State<'_, AppState>, query: String) -> Result<serde_json::Value, String> {
    with_view(&state, |e, i| e.search(i, &query))
}

#[tauri::command]
pub fn get_why(state: State<'_, AppState>, input: String) -> Result<serde_json::Value, String> {
    let idx = ensure_index(&state)?;
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let chains = engine.why(&idx, &input).map_err(|e| e)?;
    serde_json::to_value(chains).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_owners(
    state: State<'_, AppState>,
    option_path: String,
) -> Result<serde_json::Value, String> {
    let idx = ensure_index(&state)?;
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let entry = engine.owners(&idx, &option_path).map_err(|e| e)?;
    serde_json::to_value(entry).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_dependents(state: State<'_, AppState>, input: String) -> Result<serde_json::Value, String> {
    let idx = ensure_index(&state)?;
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let deps = engine.dependents(&idx, &input).map_err(|e| e)?;
    serde_json::to_value(deps).map_err(|e| e.to_string())
}
