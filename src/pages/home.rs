use dioxus::prelude::*;
use serde_json::{Value::Null};
use simpics_shared::ScreenshotEntry;

#[css_module("/src/pages/home.css")]
struct Styles;

#[component]
pub fn Home() -> Element {
    let mut status = use_signal(|| None::<String>);
    let mut screenshot_list = use_signal(|| None::<Vec<ScreenshotEntry>>);

    let loader = use_resource(|| async {
        crate::utils::call::call::<Vec<ScreenshotEntry>>("get_full_table", Null).await
    });
    match &*loader.read() {
        Some(Ok(list)) => screenshot_list.set(Some(list.clone())),
        Some(Err(e)) => status.set(Some(e.clone())),
        None => {}
    };

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
