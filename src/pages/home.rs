use dioxus::prelude::*;
use serde_json::json;

#[css_module("/src/pages/home.css")]
struct Styles;

const SCREENSHOTS_KEY: &str = "screenshots_dir";

async fn get_screenshot_dir() -> Result<String, String> {
    crate::utils::call::call(
        "get_store_value",
        json!({ "key": SCREENSHOTS_KEY }),
    )
    .await
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
    
    

    rsx! {
        div { class: "container",
            h1 { "Home" }

            if loader.read().is_none() {
                p { "Loading..." }
            } else if let Some(msg) = status() {
                p { "{msg}" }
            } else {

            }
        }
    }
}