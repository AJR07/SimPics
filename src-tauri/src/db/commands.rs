use crate::{
    db::{
        screenshot_schema::{ Column, Entity, Model, OptionalModel },
        utils::{ DB_CONN, DbError, conn, full_update_frontend },
    },
    fs,
};
use sea_orm::{
    ColumnTrait,
    QueryFilter,
    Database,
    DatabaseConnection,
    EntityTrait,
    IntoActiveModel,
    ModelTrait,
    sea_query::Expr,
};
use tauri::{ AppHandle };

pub async fn init(app: AppHandle) -> Result<DatabaseConnection, Box<dyn std::error::Error>> {
    let mut full_path = dirs::data_local_dir().unwrap();
    full_path.push("simpics");

    // create directories if not exist
    std::fs::create_dir_all(&full_path)?;

    full_path.push("data.db");
    let full_path_str = full_path.to_str().unwrap().to_string();

    // connect to db
    tracing::info!("{}", format!("Directory: sqlite:///{}?mode=rwc", full_path_str));
    let db: DatabaseConnection = Database::connect(
        format!("sqlite:///{}?mode=rwc", full_path_str)
    ).await?;

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
        full_update_frontend().await?;
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

    // update frontend
    full_update_frontend().await?;
    Ok(())
}

#[tauri::command]
pub async fn update_entry(
    old_name: String,
    updated: OptionalModel,
) -> Result<bool, DbError> {
    let db_conn = conn()?;

    let mut query = Entity::update_many()
        .filter(Column::Name.eq(old_name));

    let mut has_updates = false;

    if let Some(name) = updated.name {
        query = query.col_expr(Column::Name, Expr::value(name));
        has_updates = true;
    }

    if let Some(date) = updated.date {
        query = query.col_expr(Column::Date, Expr::value(date));
        has_updates = true;
    }

    if let Some(path) = updated.path {
        query = query.col_expr(Column::Path, Expr::value(path));
        has_updates = true;
    }

    // Avoid executing an update with no columns.
    if !has_updates {
        return Err(DbError::NoCommandedUpdate);
    }

    let result = query.exec(db_conn).await?;

    if result.rows_affected == 1 {
        full_update_frontend().await?;
        Ok(true)
    } else {
        Ok(false)
    }
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
        Err(e) =>
            tracing::info!(
                "{}",
                format!("Attempted to close database but encountered an error: {e}")
            ),
    }
    Ok(())
}
