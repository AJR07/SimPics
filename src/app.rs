#![allow(non_snake_case)]

use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

static CSS: Asset = asset!("/assets/styles.css");
static DX_THEME: Asset = asset!("/assets/dx-components-theme.css");

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"], js_name = invoke)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[component]
pub fn App() -> Element {
    rsx! {
        link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        link {
            rel: "preconnect",
            href: "https://fonts.gstatic.com",
            crossorigin: "anonymous", // or crossorigin: true / "true" depending on your target requirements
        }
        link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Nunito:ital,wght@0,200..1000;1,200..1000&family=Rubik:ital,wght@0,300..900;1,300..900&display=swap",
        }
        link { rel: "stylesheet", href: CSS }
        link { rel: "stylesheet", href: DX_THEME }
        main { crate::pages::settings::Settings {} }
    }
}
