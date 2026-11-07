use crate::{ db::commands::get_full_table, APP_HANDLE };
use sea_orm::DatabaseConnection;
use serde::Serialize;
use tauri::Emitter;
use std::sync::OnceLock;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database connection not initialized")]
    NotInitialized,
    #[error("app not avail")]
    AppHandleNotAvail,
    #[error("multiple rows deleted")]
    MultipleRowsDeleted,
    #[error("no update commanded")]
    NoCommandedUpdate,
    #[error("failed to emit frontend event: {0}")] 
    EventEmit(String),
    #[error("database error: {0}")] Db(#[from] sea_orm::DbErr),
}

// Tauri needs errors to be serializable to send them to the frontend
impl Serialize for DbError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

// * handler for DB_CONN
pub static DB_CONN: OnceLock<DatabaseConnection> = OnceLock::new();

pub fn conn() -> Result<&'static DatabaseConnection, DbError> {
    DB_CONN.get().ok_or(DbError::NotInitialized)
}

// * handler for app_handle

fn app_handle() -> Result<&'static tauri::AppHandle, DbError> {
    APP_HANDLE.get().ok_or(DbError::AppHandleNotAvail)
}

pub async fn full_update_frontend() -> Result<(), DbError> {
    let app_handle = app_handle()?;
    let full_table = get_full_table().await?;
    app_handle
        .emit("new_full_db", full_table)
        .map_err(|error| DbError::EventEmit(error.to_string()))?;
    Ok(())
}
