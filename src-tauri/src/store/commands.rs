use tauri::{AppHandle};
use tauri_plugin_store::StoreExt;

#[tauri::command]
pub fn get_store_value(
    app: AppHandle,
    key: String,
) -> Result<serde_json::Value, String> {
    let store = app
        .store("settings.json")
        .map_err(|e| e.to_string())?;

    store
        .get(&key)
        .ok_or_else(|| format!("Key '{}' not found in the store (get)", key))
}

#[tauri::command]
pub fn get_store_value_with_default(
    app: AppHandle,
    key: String,
    default: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let store = app
        .store("settings.json")
        .map_err(|e| e.to_string())?;

    match store.get(&key) {
        Some(value) => Ok(value),
        None => {
            store.set(key, default.clone());
            store.save().map_err(|e| e.to_string())?;
            Ok(default)
        }
    }
}

#[tauri::command]
pub fn set_store_value(
    app: AppHandle,
    key: String,
    value: serde_json::Value,
) -> Result<bool, String> {
    let store = app
        .store("settings.json")
        .map_err(|e| e.to_string())?;

    store.set(key, value);

    store.save().map_err(|e| e.to_string())?;

    Ok(true)
}