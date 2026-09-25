#![allow(non_snake_case)]

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

static CSS: Asset = asset!("/assets/styles.css");

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub fn App() -> Element {
    rsx! {
        link { rel: "stylesheet", href: CSS }
        main { class: "container",
            h1 { "Welcome to Tauri + Dioxus :D" }
        
        }
    }
}
