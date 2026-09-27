mod store;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            store::commands::get_store_value,
            store::commands::set_store_value,
            store::commands::get_store_value_with_default
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
