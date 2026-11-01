use dioxus::prelude::*;
use serde_json::Value::Null;
use simpics_shared::ScreenshotEntry;
use wasm_bindgen::{JsValue, prelude::Closure};

use crate::utils::{call::call, listen::listen};

#[css_module("/src/pages/home.css")]
struct Styles;

#[component]
pub fn Home() -> Element {
    let mut status = use_signal(|| None::<String>);
    let mut screenshot_list = use_signal(|| None::<Vec<ScreenshotEntry>>);

        use_hook(|| {
        let handler = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
            let payload = js_sys::Reflect::get(&event, &JsValue::from_str("payload"))
                .unwrap_or(JsValue::NULL);

            match serde_wasm_bindgen::from_value::<Vec<ScreenshotEntry>>(payload) {
                Ok(list) => screenshot_list.set(Some(list)),
                Err(e) => status.set(Some(format!("bad new_full_db payload: {e}"))),
            }
        });

        spawn(async move {
            // listen first, so no update can slip through during the initial load
            if let Err(e) = listen("new_full_db", &handler).await {
                status.set(Some(format!("couldn't initiate listener: {e:?}")));
            }
            handler.forget();

            // initial load, once
            match call::<Vec<ScreenshotEntry>>("get_full_table", Null).await {
                Ok(list) => {
                    // don't clobber a newer list if an event already arrived
                    if screenshot_list.peek().is_none() {
                        screenshot_list.set(Some(list));
                    }
                }
                Err(e) => status.set(Some(e)),
            }
        });
    });

    rsx! {
        div {
            h1 { "Home" }

            if let Some(msg) = status() {
                p { "{msg}" }
            }
            if let Some(list) = screenshot_list() {
                p { "Number of Rows: {list.len()}" }
                for screenshot_entry in list.iter() {
                    p { "Name: {screenshot_entry.name}, Date: {screenshot_entry.date}" }
                }
            }
        }
    }
}
