use chrono::{DateTime, Utc};
use std::fs;
use tauri::AppHandle;

use crate::db::schema::Model;
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
