use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::fs;
use tauri::AppHandle;

use crate::db::{
    commands,
    screenshot_schema::{Model, OptionalModel},
};

use crate::store;

pub fn full_scan(app: AppHandle) -> Result<Vec<Model>, String> {
    let raw_screenshot_dir =
        match store::commands::get_store_value(app, "screenshots_dir".to_string()) {
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

    for path in paths {
        let path_converted = path.unwrap().path();
        let file_name = path_converted.to_str().unwrap();

        // TODO: SUPPORT PARSING DATETIME FROM FILE NAME WITH UTC INPUT
        // retrieve date using the file's creation date (system-time => UTC)
        let file_created_date: DateTime<Utc> =
            match fs::metadata(&file_name).and_then(|m| m.created()) {
                Ok(created) => DateTime::<Utc>::from(created),
                Err(e) => {
                    tracing::error!("Cannot get created time from metadata: {e}");
                    // TODO: default for now is current time
                    Utc::now()
                }
            };

        let new_screenshot = Model {
            name: file_name.to_string(),
            date: file_created_date,
        };

        res.push(new_screenshot);
    }

    Ok(res)
}

pub async fn apply_full_changes(app: AppHandle) -> Result<(), String> {
    // Complete a full scan (use `?` instead of unwrap so a failed scan doesn't panic)
    let file_list = full_scan(app)?;

    // Get old data
    let old_data = crate::db::commands::get_full_table()
        .await
        .map_err(|e| e.to_string())?;

    // Index the DB rows by file name
    let mut old_map: HashMap<String, Model> =
        old_data.into_iter().map(|m| (m.name.clone(), m)).collect();

    let mut to_insert: Vec<Model> = Vec::new();
    let mut to_update: Vec<(String, OptionalModel)> = Vec::new();

    for scanned in file_list {
        match old_map.remove(&scanned.name) {
            // 1. In the scan, not in the DB -> new file
            None => to_insert.push(scanned),

            // 2. In both, but the date differs -> updated
            // (compare in seconds, see note below)
            Some(old) if old.date.timestamp() != scanned.date.timestamp() => {
                to_update.push((
                    old.name,
                    OptionalModel {
                        date: Some(scanned.date),
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
    for entry in to_insert.iter() {
        if let Err(e) = commands::insert_entry(entry.clone()).await {
            tracing::error!("Cannot insert entry due to: {}", e);
            return Err(e.to_string());
        }
    }

    for entry in to_update.iter() {
        match commands::update_entry(entry.clone().0, entry.clone().1).await {
            Ok(true) => {}
            Ok(false) => {
                tracing::error!("Entry wasn't found in database");
                return Err("Entry wasn't found in database".to_string());
            }
            Err(e) => {
                tracing::error!("Cannot update entry due to: {}", e);
                return Err(e.to_string());
            }
        }
    }

    for entry in to_delete.iter() {
        match commands::delete_entry(entry.clone().name).await {
            Ok(true) => {}
            Ok(false) => {
                tracing::error!("Entry wasn't found in database");
                return Err("Entry wasn't found in database".to_string());
            }
            Err(e) => {
                tracing::error!("Cannot delete entry due to: {}", e);
                return Err(e.to_string());
            }
        }
    }

    Ok(())
}
