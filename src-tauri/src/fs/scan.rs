use std::collections::HashMap;
use std::fs;
use tauri::AppHandle;

use crate::{
    db::{ commands, screenshot_schema::{ Model, OptionalModel } },
    fs::utils::{ get_file_details, is_image_file },
};

use crate::store;

pub fn full_scan(app: AppHandle) -> Result<Vec<Model>, String> {
    let raw_screenshot_dir = match
        store::commands::get_store_value(app, "screenshots_dir".to_string())
    {
        Ok(r) => {
            tracing::info!("{}", format!("Full scan on directory: {}", r));
            r
        }
        Err(e) => {
            tracing::error!("Cannot get screenshots_dir");
            return Err(e.to_string());
        }
    };
    let screenshot_dir = match raw_screenshot_dir.as_str() {
        Some(r) => r,
        None => {
            tracing::error!("Cannot convert screenshots_dir to str");
            return Err("Cannot convert screenshots_dir to str".to_string());
        }
    };

    let paths = match fs::read_dir(screenshot_dir) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("{}", format!("Cannot retrieve files in directory: {e}"));
            return Err(e.to_string());
        }
    };

    // create object
    let mut res: Vec<Model> = Vec::new();

    for dir_entry in paths {
        let entry = match dir_entry {
            Ok(e) => e,
            Err(e) => {
                tracing::error!("Cannot read directory entry: {e}");
                continue;
            }
        };
        let path = entry.path();

        if !path.is_file() {
            tracing::debug!("Skipping non-file: {path:?}");
            continue;
        }

        if !is_image_file(&path) {
            tracing::debug!("Skipping non-image: {path:?}");
            continue;
        }

        match get_file_details(&path) {
            Ok(model) => res.push(model),
            Err(e) => tracing::error!("Cannot read file details for {path:?}: {e}"),
        }
    }

    Ok(res)
}

pub async fn apply_full_changes(app: AppHandle) -> Result<(), String> {
    // Complete a full scan - using spawn blocking for async results
    let file_list = tauri::async_runtime
        ::spawn_blocking(move || full_scan(app)).await
        .map_err(|e| e.to_string())??;

    // Get old data
    let old_data = crate::db::commands::get_full_table().await.map_err(|e| e.to_string())?;

    // Index the DB rows by file name
    let mut old_map: HashMap<String, Model> = old_data
        .into_iter()
        .map(|m| (m.name.clone(), m))
        .collect();

    let mut to_insert: Vec<Model> = Vec::new();
    let mut to_update: Vec<(String, OptionalModel)> = Vec::new();

    for scanned in file_list {
        match old_map.remove(&scanned.name) {
            // 1. In the scan, not in the DB -> new file
            None => to_insert.push(scanned),

            // 2. In both, but the date differs -> updated
            // (compare in seconds, see note below)
            Some(old) if
                old.date.timestamp() != scanned.date.timestamp() ||
                old.path != scanned.path
            => {
                to_update.push((
                    old.name,
                    OptionalModel {
                        date: Some(scanned.date),
                        path: Some(scanned.path),
                        name: None,
                    },
                ));
            }

            // 4. In both and identical -> nothing to do
            Some(_) => {}
        }
    }

    // 3. Anything still in old_map was in the DB but not in the scan -> deleted
    let to_delete: Vec<Model> = old_map.into_values().collect();

    tracing::info!(
        "Scan diff: {} new, {} updated, {} deleted",
        to_insert.len(),
        to_update.len(),
        to_delete.len()
    );

    // * Apply to DB
    let mut failures = 0usize;

    for entry in to_insert {
        if let Err(e) = commands::insert_entry(entry.clone()).await {
            tracing::error!("Cannot insert entry due to: {e}");
            failures += 1;
        } else {
            tracing::warn!("Created {:?}", entry.name);
        }
    }

    for (name, patch) in to_update {
        match commands::update_entry(name.clone(), patch.clone()).await {
            Ok(true) => tracing::warn!("Updated {patch:?}"),
            Ok(false) => tracing::warn!("Update skipped, {name} is no longer in the DB"),
            Err(e) => {
                tracing::error!("Cannot update entry {name} due to: {e}");
                failures += 1;
            }
        }
    }

    for entry in to_delete {
        let name = entry.name;
        match commands::delete_entry(name.clone()).await {
            Ok(true) => tracing::warn!("Deleted {name}"),
            Ok(false) => tracing::warn!("Delete skipped, {name} is already gone from the DB"),
            Err(e) => {
                tracing::error!("Cannot delete entry {name} due to: {e}");
                failures += 1;
            }
        }
    }

    if failures > 0 {
        return Err(format!("{failures} DB operations failed during full scan"));
    }

    Ok(())
}
