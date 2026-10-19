use std::fs;
use tauri::AppHandle;

use crate::store;

pub fn full_scan(app: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
	let screenshot_dir = match store::commands::get_store_value(app, "screenshots_dir".to_string()) {
        Ok(r) => {tracing::info!("{}", format!("Full scan on directory: {}", r)); r},
        Err(e) => {tracing::error!("Cannot get screenshots_dir"); return Err(e.into());}
    };
	let paths = fs::read_dir(screenshot_dir.to_string()).unwrap();

    for path in paths {
        println!("Name: {}", path.unwrap().path().display())
    }

    Ok(())
}