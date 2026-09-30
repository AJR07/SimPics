#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

static CSS: Asset = asset!("/assets/styles.css");

const SCREENSHOTS_KEY: &str = "screenshots_dir";
const DEFAULT_SCREENSHOTS_DIR: &str = "C:\\Users\\User\\Pictures\\Screenshots";

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

/// Calls a Tauri command and converts every failure mode into a `String` error.
async fn call<T: DeserializeOwned>(cmd: &str, args: Value) -> Result<T, String> {
    let args = to_value(&args).map_err(|e| format!("Serialize error: {e}"))?;

    let result = invoke(cmd, args)
        .await
        .map_err(|e| e.as_string().unwrap_or_else(|| "Unknown error".into()))?;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("Deserialize error: {e}"))
}

async fn get_screenshot_dir() -> Result<String, String> {
    call(
        "get_store_value_with_default",
        json!({ "key": SCREENSHOTS_KEY, "default": DEFAULT_SCREENSHOTS_DIR }),
    )
    .await
}

async fn set_screenshot_dir(value: String) -> Result<(), String> {
    // Ignore whatever the command returns; we only care whether it succeeded.
    call::<Value>("set_store_value", json!({ "key": SCREENSHOTS_KEY, "value": value }))
        .await
        .map(|_| ())
}

#[component]
pub fn App() -> Element {
    let mut screenshot_dir = use_signal(String::new);
    let mut status = use_signal(|| None::<String>);

    // Load the stored value once and copy it into the editable signal.
    let loader = use_resource(get_screenshot_dir);
    use_effect(move || match &*loader.read() {
        Some(Ok(dir)) => screenshot_dir.set(dir.clone()),
        Some(Err(e)) => status.set(Some(e.clone())),
        None => {}
    });

    let is_loading = loader.read().is_none();

    rsx! {
        link { rel: "stylesheet", href: CSS }
        main { class: "container",
            h1 { "Settings" }

            if is_loading {
                p { "Loading..." }
            }
            if let Some(msg) = status() {
                p { "{msg}" }
            }

            form {
                onsubmit: move |e| {
                    e.prevent_default();
                    let value = screenshot_dir();
                    spawn(async move {
                        match set_screenshot_dir(value).await {
                            Ok(()) => status.set(Some("Saved!".into())),
                            Err(e) => status.set(Some(format!("Save failed: {e}"))),
                        }
                    });
                },
                input {
                    value: "{screenshot_dir}",
                    disabled: is_loading,
                    oninput: move |e| screenshot_dir.set(e.value()),
                }
                button { r#type: "submit", disabled: is_loading, "Save" }
            }
        }
    }
}