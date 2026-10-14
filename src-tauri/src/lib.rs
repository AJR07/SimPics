mod fs;
mod store;

const SCREENSHOTS_KEY: &str = "screenshots_dir";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            store::commands::get_store_value,
            store::commands::set_store_value,
            store::commands::get_store_value_with_default
        ])
        .setup(|app| {
            let handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
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