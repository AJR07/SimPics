use sea_orm::{Database, DatabaseConnection};

pub async fn init() -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
    let mut full_path = dirs::data_local_dir().unwrap();
    full_path.push("simpics");
    std::fs::create_dir_all(&full_path)?;

    full_path.push("data.db");
    let full_path_str = full_path.to_str().unwrap().to_string();

    tracing::info!("{}", format!("Directory: sqlite:///{}?mode=rwc", full_path_str));
    let db: DatabaseConnection =
        Database::connect(format!("sqlite:///{}?mode=rwc", full_path_str)).await?;

    Ok(db)
}

pub async fn clean_and_close(db: DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    // Closing connection here
    db.close().await?;
    Ok(())
}
