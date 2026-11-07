use std::path::Path;
use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use tauri_plugin_dialog::DialogExt;

use crate::fs::{ scan::apply_full_changes, watch::{ unwatch, watch } };

//* 3 Core Commands

#[tauri::command]
pub fn get_store_value(app: AppHandle, key: String) -> Result<serde_json::Value, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    store.get(&key).ok_or_else(|| format!("Key '{}' not found in the store (get)", key))
}

#[tauri::command]
pub fn get_store_value_with_default(
    app: AppHandle,
    key: String,
    default: serde_json::Value
) -> Result<serde_json::Value, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

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
    value: serde_json::Value
) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;

    store.set(key, value);

    store.save().map_err(|e| e.to_string())?;

    Ok(())
}

//* Sub-commands

// ^ Screenshots
const SCREENSHOTS_KEY: &str = "screenshots_dir";
const DEFAULT_SCREENSHOTS_DIR: &str = "C:\\Users\\User\\Pictures\\Screenshots";

#[derive(Serialize)]
pub struct ScreenshotResult {
    valid: bool,
    cur_path: String,
}

#[tauri::command]
pub fn get_screenshot_dir(app: AppHandle) -> Result<String, String> {
    let default_dir = serde_json::Value::String(DEFAULT_SCREENSHOTS_DIR.to_string());
    let res = get_store_value_with_default(app, SCREENSHOTS_KEY.to_string(), default_dir)?;

    res.as_str()
        .map(str::to_owned)
        .ok_or_else(|| "Stored screenshot directory must be a string".to_string())
}

// use the folder directory window to pick a directory
#[tauri::command]
pub async fn req_screenshot_dir(app: AppHandle) -> Result<ScreenshotResult, String> {
    let Some(new_path) = app
        .dialog()
        .file()
        .set_title("Select a folder")
        .blocking_pick_folder()
        .map(|path| path.to_string()) else {
        return Ok(ScreenshotResult {
            valid: false,
            cur_path: get_screenshot_dir(app)?,
        });
    };

    let path = Path::new(&new_path);

    if !path.try_exists().map_err(|e| e.to_string())? || !path.is_dir() {
        return Ok(ScreenshotResult {
            valid: false,
            cur_path: get_screenshot_dir(app)?,
        });
    }

    // ^ All OK, set store value
    tracing::warn!("Screenshot Directory Changed, reconfiguring: {:?}", new_path.clone());
    set_store_value(
        app.clone(),
        SCREENSHOTS_KEY.to_string(),
        serde_json::Value::String(new_path.clone())
    )?;
    tracing::warn!("Store value set successfully!");

    // ^ Then restart watch and scan
    apply_full_changes(app.clone()).await?;
    tracing::warn!("Full scan completed successfully!");

    // ^ Then restart watch
    unwatch(app.clone())?;
    watch(app.clone(), new_path.clone()).await.map_err(|e| e.to_string())?;

    Ok(ScreenshotResult {
        valid: true,
        cur_path: new_path,
    })
}
