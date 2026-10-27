use crate::{
    db::screenshot_schema::{ActiveModel, Entity, Model, OptionalModel}, fs,
};
use sea_orm::{ActiveValue::Set, Database, DatabaseConnection, EntityTrait, IntoActiveModel, ModelTrait};
use serde::Serialize;
use std::sync::OnceLock;
use tauri::AppHandle;
use sea_orm::ActiveModelTrait;

static DB_CONN: OnceLock<DatabaseConnection> = OnceLock::new();

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database connection not initialized")]
    NotInitialized,
    #[error("multiple rows deleted")]
    MultipleRowsDeleted,
    #[error("object not found")]
    EntryNotFound,
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
    fs::scan::apply_full_changes(app).await?; // for now, unwrap

    Ok(db)
}

// If entry not found, returned as None
#[tauri::command]
pub async fn get_entry(name: String) -> Result<Option<Model>, DbError> {
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

#[tauri::command]
pub async fn insert_entry(new_entry: Model) -> Result<(), DbError> {
    let db_conn = conn()?;
    let active_new_entry = new_entry.into_active_model();
    Entity::insert(active_new_entry).exec(db_conn).await?;
    Ok(())
}

#[tauri::command]
pub async fn update_entry(old_name: String, updated_entry: OptionalModel) -> Result<bool, DbError> {
    let db_conn = conn()?;
    let mut old_entry: ActiveModel = match get_entry(old_name).await? {
        Some(r) => r.into(),
        None => return Err(DbError::EntryNotFound)
    };
    
    // update old entry with new changes
    if let Some(r) = updated_entry.name {  
        old_entry.name = Set(r);
    }

    if let Some(r) = updated_entry.date {  
        old_entry.date = Set(r);
    }

    old_entry.update(db_conn).await?;

    Ok(true)
}

#[tauri::command]
pub async fn get_full_table() -> Result<Vec<Model>, DbError> {
    let db_conn = conn()?;
    let res: Vec<Model> = Entity::find().all(db_conn).await?;
    Ok(res)
}

pub async fn close() -> Result<(), DbError> {
    let db_conn = conn()?.clone();
    // Closing connection here
    match db_conn.close().await {
        Ok(()) => tracing::info!("Database closed successfully"),
        Err(e) => tracing::info!(
            "{}",
            format!("Attempted to close database but encountered an error: {e}")
        ),
    }
    Ok(())
}
