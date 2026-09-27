#![allow(non_snake_case)]

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use serde_wasm_bindgen::to_value;
use serde_json::json;

static CSS: Asset = asset!("/assets/styles.css");

#[wasm_bindgen]
extern "C" {
    // invoke without arguments
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn invoke_without_args(cmd: &str) -> JsValue;

    // invoke with arguments (default)
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[component]
pub fn App() -> Element {
    let screenshot_dir = use_resource(move || async move {
        let args = to_value(&json!({ "key": "screenshots_dir", "default": "C:\\Users\\User\\Pictures\\Screenshots" })).unwrap();

        match invoke("get_store_value_with_default", args).await {
            Ok(result) => serde_wasm_bindgen::from_value::<String>(result)
                .unwrap_or_else(|e| format!("Deserialize error: {e}")),
            Err(e) => {
                // e is a JsValue — usually the string you returned from Err(...) in Rust
                let msg = e.as_string().unwrap_or_else(|| "Unknown error".to_string());
                format!("Store error: {msg}")
            }
        }
    });

    rsx! {
        link { rel: "stylesheet", href: CSS }
        main { class: "container",
            h1 { "HALLOOOOOOOOOOOOOOOOOOO :D" }
            p {
                match &*screenshot_dir.read() {
                    Some(dir) => rsx! { "{dir}" },
                    None => rsx! { "Loading..." },
                }
            }
        }
    }
}