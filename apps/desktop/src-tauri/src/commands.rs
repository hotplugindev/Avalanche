use avalanche_model::{Host, MutationIntent, Transaction};
use serde::Serialize;

#[derive(Serialize)]
pub struct RepositoryInfo {
    pub root: String,
    pub schema_version: Option<u32>,
}

#[derive(Serialize)]
pub struct InspectResult {
    pub path: String,
    pub definitions_count: usize,
}

#[tauri::command]
pub fn get_repository_info() -> Result<RepositoryInfo, String> {
    Ok(RepositoryInfo {
        root: String::new(),
        schema_version: None,
    })
}

#[tauri::command]
pub fn get_hosts() -> Result<Vec<Host>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub fn get_options() -> Result<Vec<String>, String> {
    Ok(Vec::new())
}

#[tauri::command]
pub fn inspect_option(path: String) -> Result<InspectResult, String> {
    Ok(InspectResult {
        path,
        definitions_count: 0,
    })
}

#[tauri::command]
pub fn plan_mutation(intents: Vec<MutationIntent>) -> Result<Transaction, String> {
    let txn = Transaction::new("txn-ui", intents);
    Ok(txn)
}

#[tauri::command]
pub fn apply_transaction(_transaction_id: String) -> Result<(), String> {
    Err("not yet implemented".to_string())
}
