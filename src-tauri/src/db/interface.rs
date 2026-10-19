use std::sync::OnceLock;

use sea_orm::{Database, DatabaseConnection};
use tauri::AppHandle;

use crate::{db::schema, fs};

static DB_CONN: OnceLock<DatabaseConnection> = OnceLock::new();

pub async fn init(app: AppHandle) -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
    let mut full_path = dirs::data_local_dir().unwrap();
    full_path.push("simpics");

    // create directories if not exist
    std::fs::create_dir_all(&full_path)?;

    full_path.push("data.db");
    let full_path_str = full_path.to_str().unwrap().to_string();

    // connect to db
    tracing::info!(
        "{}",
        format!("Directory: sqlite:///{}?mode=rwc", full_path_str)
    );
    let db: DatabaseConnection =
        Database::connect(format!("sqlite:///{}?mode=rwc", full_path_str)).await?;
    DB_CONN.set(db.to_owned()).unwrap();

    // sync entity schema
    db.get_schema_builder().register(schema::Entity).sync(&db).await?;

    // scan and implement changes
    fs::scan::full_scan(app);

    Ok(db)
}

pub async fn clean_and_close(db: DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    // Closing connection here
    db.close().await?;
    Ok(())
}
