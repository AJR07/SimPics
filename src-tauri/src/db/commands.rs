use std::sync::OnceLock;
use sea_orm::{Database, DatabaseConnection, EntityTrait, ModelTrait};
use tauri::AppHandle;
use serde::Serialize;
use crate::{db::schema::{Entity, Model}, fs};

static DB_CONN: OnceLock<DatabaseConnection> = OnceLock::new();

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database connection not initialized")]
    NotInitialized,
    #[error("multiple rows deleted")]
    MultipleRowsDeleted,
    #[error("database error: {0}")]
    Db(#[from] sea_orm::DbErr),
}

// Tauri needs errors to be serializable to send them to the frontend
impl Serialize for DbError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

fn conn() -> Result<&'static DatabaseConnection, DbError> {
    DB_CONN.get().ok_or(DbError::NotInitialized)
}

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

    // lock db connection
    DB_CONN.set(db.to_owned()).unwrap();

    // sync entity schema
    db.get_schema_builder().register(Entity).sync(&db).await?;

    // scan and implement changes
    fs::scan::full_scan(app);

    Ok(db)
}

// If entry not found, returned as None
#[tauri::command]
pub async fn get_entry(name: String) -> Result<Option<Model>, DbError>  {
    let db_conn = conn()?;
    let res: Option<Model> = Entity::find_by_id(name).one(db_conn).await?;
    Ok(res)
}

#[tauri::command]
pub async fn delete_entry(name: String) -> Result<bool, DbError> {
    // get the entry first, handle errors appropriately
    let Some(model) = get_entry(name).await? else {
        return Ok(false);
    };

    let db_conn = conn()?;

    let res = model.delete(db_conn).await?;
    
    if res.rows_affected <= 1 {
        Ok(res.rows_affected == 1)
    } else {
        Err(DbError::MultipleRowsDeleted)
    }
}

// #[tauri::command]
// pub async fn insert_entry(name: String) -> Result<Option<Model>, Box<dyn std::error::Error>>  {
//     let db_conn = conn()?;
//     let res: Option<Model> = match Entity::find_by_id(name.clone()).one(db_conn).await {
//         Ok(r) => r,
//         Err(e) => {tracing::debug!("{}", format!("Couldn't get entry with name: {name} due to error: {e}")); return Err(Box::new(e))}
//     };
//     Ok(res)
// }

// #[tauri::command]
// pub async fn update_entry(name: String) -> Result<Option<Model>, Box<dyn std::error::Error>>  {
//     let db_conn = conn()?;
//     let res: Option<Model> = match Entity::find_by_id(name.clone()).one(db_conn).await {
//         Ok(r) => r,
//         Err(e) => {tracing::debug!("{}", format!("Couldn't get entry with name: {name} due to error: {e}")); return Err(Box::new(e))}
//     };
//     Ok(res)
// }

// #[tauri::command]
// pub async fn get_full_table(name: String) -> Result<Option<Model>, Box<dyn std::error::Error>>  {
//     let db_conn = conn()?;
//     let res: Option<Model> = match Entity::find_by_id(name.clone()).one(db_conn).await {
//         Ok(r) => r,
//         Err(e) => {tracing::debug!("{}", format!("Couldn't get entry with name: {name} due to error: {e}")); return Err(Box::new(e))}
//     };
//     Ok(res)
// }

pub async fn close(db: DatabaseConnection) -> Result<(), Box<dyn std::error::Error>> {
    // Closing connection here
    db.close().await?;
    Ok(())
}
