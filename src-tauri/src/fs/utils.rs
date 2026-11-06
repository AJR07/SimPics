use chrono::{DateTime, Utc};
use image::ImageReader;
use std::{fs, path::Path};

use crate::db::screenshot_schema::Model;

pub fn get_file_details(file_path: &Path) -> Result<Model, String> {
    let metadata = fs
        ::metadata(&file_path)
        .map_err(|e| format!("Cannot read metadata for {}: {e}", file_path.display()))?;

    let name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Cannot get a valid UTF-8 file name".to_string())?
        .to_string();

    let date = metadata.created().map_err(|e| format!("Cannot read creation time: {e}"))?;

    let date_utc: DateTime<Utc> = date.into();

    Ok(Model {
        name,
        date: date_utc,
        path: file_path.to_string_lossy().to_string(),
    })
}

pub fn file_name_string(p: &Path) -> Option<String> {
    p.file_name()
        .and_then(|n| n.to_str())
        .map(str::to_owned)
}

pub fn is_image_file(p: &Path) -> bool {
	ImageReader::open(p)
        .and_then(|r| r.with_guessed_format())
        .map(|r| r.format().is_some())
        .unwrap_or(false)
}