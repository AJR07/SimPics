use dioxus::prelude::*;
use serde::Deserialize;
use serde_json::json;
use wasm_bindgen::prelude::*;

#[css_module("/src/pages/home.css")]
struct Styles;

const SCREENSHOTS_KEY: &str = "screenshots_dir";

async fn get_screenshot_dir() -> Result<String, String> {
    crate::utils::call::call("get_store_value", json!({ "key": SCREENSHOTS_KEY })).await
}

#[derive(Deserialize)]
struct TauriEvent<T> {
    payload: T,
}

#[component]
pub fn Home() -> Element {
    let mut screenshot_dir = use_signal(String::new);
    let mut status = use_signal(|| None::<String>);

    // Load the stored value once and copy it into the editable signal.
    let loader = use_resource(get_screenshot_dir);
    use_effect(move || match &*loader.read() {
        Some(Ok(dir)) => screenshot_dir.set(dir.clone()),
        Some(Err(e)) => status.set(Some(e.clone())),
        None => {}
    });

    // listen for changes
    use_future(move || async move {
        let cb = Closure::<dyn FnMut(JsValue)>::new(move |ev: JsValue| {
            if let Ok(e) = serde_wasm_bindgen::from_value::<TauriEvent<String>>(ev) {
                status.set(Some(e.payload));
            }
        });

        match crate::utils::listen::listen("file-watcher_change", &cb).await {
            Ok(_) => cb.forget(),
            Err(e) => {
                status.set(Some(format!(
                    "Failed to listen for file changes: {}",
                    e.as_string().unwrap_or_else(|| "unknown error".into())
                )));
            }
        }
    });

    rsx! {
        div { class: "container",
            h1 { "Home" }

            if loader.read().is_none() {
                p { "Loading..." }
            } else if let Some(msg) = status() {
                p { "{msg}" }
            } else {
                p { "HI" }
            }
        }
    }
}
