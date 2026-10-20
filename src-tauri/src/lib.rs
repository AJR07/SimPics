mod db;
mod fs;
mod store;
use std::sync::OnceLock;

const SCREENSHOTS_KEY: &str = "screenshots_dir";

static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    //* initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            store::commands::get_store_value,
            store::commands::set_store_value,
            store::commands::get_store_value_with_default,
            db::commands::get_entry,
            db::commands::delete_entry
        ])
        .setup(|app| {
            // lock app handle
            APP_HANDLE.set(app.handle().to_owned()).unwrap();

            let handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                // * initialize database
                match db::commands::init(handle.clone()).await {
                    Ok(r) => {
                        tracing::info!("Database initialized successfully.");
                        r
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize database: {}", e);
                        panic!("{}", "Unrecoverable - DB")
                    }
                };

                let path = match store::commands::get_store_value(
                    handle.clone(),
                    SCREENSHOTS_KEY.to_string(),
                ) {
                    Ok(serde_json::Value::String(path)) => path,
                    Ok(value) => {
                        println!(
                            "Error retrieving screenshots_dir: expected a string, got {value}"
                        );
                        return;
                    }
                    Err(e) => {
                        println!("Error retrieving screenshots_dir: {e}");
                        return;
                    }
                };

                if let Err(e) = crate::fs::watch::async_watch(handle, path).await {
                    println!("error: {e:?}")
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
