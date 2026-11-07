use dioxus::prelude::*;
use serde_json::Value::Null;
use simpics_shared::ScreenshotEntry;
use wasm_bindgen::{ JsValue, prelude::Closure };
use std::{ cell::{ Cell, RefCell }, rc::Rc };

use crate::utils::js_cmds;

#[css_module("/src/pages/home.css")]
struct Styles;

#[component]
pub fn Home() -> Element {
    // Reactive state: writing to a signal re-renders the component
    let mut status = use_signal(|| None::<String>);
    let mut screenshot_list = use_signal(|| None::<Vec<ScreenshotEntry>>);

    // use_hook runs once and persists across renders.
    // Rc = shared ownership between closures; Cell/RefCell = mutation through Rc.
    let disposed = use_hook(|| Rc::new(Cell::new(false))); // true once unmounted
    let unlisten = use_hook(|| Rc::new(RefCell::new(None::<JsValue>))); // JS "stop listening" fn, set after listen() resolves

    // Called by JS on each `new_full_db` event.
    // Must live in a hook: a dropped Closure becomes a dead function on the JS side.
    let handler = use_hook(|| {
        Rc::new(
            Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
                let payload = js_sys::Reflect
                    ::get(&event, &JsValue::from_str("payload"))
                    .unwrap_or(JsValue::NULL);

                match serde_wasm_bindgen::from_value::<Vec<ScreenshotEntry>>(payload) {
                    Ok(list) => screenshot_list.set(Some(list)),
                    Err(error) => status.set(Some(format!("bad new_full_db payload: {error}"))),
                }
            })
        )
    });

    // ^ Cleanup on unmount.
    // Clone first, because the future below moves the originals.
    use_drop({
        let (disposed, unlisten) = (disposed.clone(), unlisten.clone());
        move || {
            disposed.set(true);
            if let Some(listener) = unlisten.borrow_mut().take() {
                let _ = js_cmds::unlisten(listener);
            }
        }
    });

    // ^ Subscribe to backend events.
    // Outer closure returns a future; clone Rcs before `async move` so the future owns its handles.
    use_future(move || {
        let (disposed, unlisten, handler) = (disposed.clone(), unlisten.clone(), handler.clone());
        async move {
            match js_cmds::listen("new_full_db", &handler).await {
                // Unmounted while awaiting: use_drop found nothing to stop, so stop it now
                Ok(listener) if disposed.get() => {
                    if let Err(error) = js_cmds::unlisten(listener) {
                        status.set(Some(error));
                    }
                }
                // Normal case: store it so use_drop can stop it later
                Ok(listener) => *unlisten.borrow_mut() = Some(listener),
                Err(error) => status.set(Some(format!("couldn't initiate listener: {error:?}"))),
            }
        }
    });

    // ^ Initial load, separate from the subscription so neither can block the other.
    // Only captures Copy signals, so no clones needed.
    use_future(move || async move {
        match js_cmds::call::<Vec<ScreenshotEntry>>("get_full_table", Null).await {
            Ok(list) => screenshot_list.set(Some(list)),
            Err(error) => status.set(Some(error)),
        }
    });

    // Recomputed each render; flatten() yields nothing while the list is None
    let img_list: Vec<String> = screenshot_list
        .read()
        .iter()
        .flatten()
        .map(|s| js_cmds::convert_file_src(&s.path))
        .collect();

    rsx! {
        div {
            h1 { "Home" }

            if let Some(msg) = status() {
                p { "{msg}" }
            }

            div { class: Styles::grid,
                for path in img_list {
                    img { class: Styles::img, src: "{path}" }
                }
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