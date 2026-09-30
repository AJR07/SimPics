use dioxus::prelude::*;
use serde::de::DeserializeOwned;
use serde_json::{Value};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// Calls a Tauri command and converts every failure mode into a `String` error.
pub async fn call<T: DeserializeOwned>(cmd: &str, args: Value) -> Result<T, String> {
    let args = to_value(&args).map_err(|e| format!("Serialize error: {e}"))?;

    let result = invoke(cmd, args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Unknown error".into()))?;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Deserialize error: {e}"))
}