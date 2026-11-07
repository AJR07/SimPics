use crate::components::{button::Button, input::Input, label::Label};
use crate::utils::js_cmds;
use dioxus::prelude::*;
use serde::Deserialize;
use serde_json::json;

#[css_module("/src/pages/settings.css")]
struct Styles;

#[derive(Clone, Copy, PartialEq)]
enum SaveIndicator {
    Success,
    Reverted,
}

#[derive(Deserialize)]
pub struct ScreenshotResult {
    valid: bool,
    cur_path: String,
}

async fn get_screenshot_dir() -> Result<String, String> {
    js_cmds::call("get_screenshot_dir", json!({})).await
}

async fn set_screenshot_dir() -> Result<ScreenshotResult, String> {
    js_cmds::call::<ScreenshotResult>("req_screenshot_dir", json!({})).await
}

#[component]
pub fn Settings() -> Element {
    let mut screenshot_dir = use_signal(String::new);
    let mut status = use_signal(|| None::<String>);
    let mut indicator = use_signal(|| None::<SaveIndicator>);
    let mut is_saving = use_signal(|| false);

    let loader = use_resource(get_screenshot_dir);

    use_effect(move || {
        match &*loader.read() {
            Some(Ok(dir)) => screenshot_dir.set(dir.clone()),
            Some(Err(error)) => status.set(Some(error.clone())),
            None => {}
        }
    });

    let is_loading = loader.read().is_none();

    rsx! {
        div { class: Styles::container,

            h1 { "Settings" }

            if is_loading {
                p { class: Styles::loading, "Loading..." }
            }

            if let Some(message) = status() {
                p { class: Styles::status_error, "{message}" }
            }

            form {
                onsubmit: move |event| {
                    event.prevent_default();

                    indicator.set(None);
                    status.set(None);
                    is_saving.set(true);

                    spawn(async move {
                        match set_screenshot_dir().await {
                            Ok(result) if result.valid => {
                                screenshot_dir.set(result.cur_path);
                                indicator.set(Some(SaveIndicator::Success));
                            }
                            Ok(result) => {
                                screenshot_dir.set(result.cur_path);
                                indicator.set(Some(SaveIndicator::Reverted));
                            }
                            Err(error) => {
                                status.set(Some(format!("Save failed: {error}")));
                            }
                        }

                        is_saving.set(false);
                    });
                },

                div { class: Styles::row,

                    Label { html_for: "screenshot-dir", "Screenshot Directory" }

                    Input {
                        class: Styles::textfield,
                        id: "screenshot-dir",
                        value: screenshot_dir,
                        disabled: true,
                    }

                    if let Some(save_indicator) = indicator() {
                        match save_indicator {
                            SaveIndicator::Success => rsx! {
                                span {
                                    class: Styles::indicator_success,
                                    role: "img",
                                    aria_label: "Screenshot directory saved",
                                    title: "Screenshot directory saved",
                                    "✓"
                                }
                            },
                            SaveIndicator::Reverted => rsx! {
                                span {
                                    class: Styles::indicator_error,
                                    role: "img",
                                    aria_label: "Directory selection cancelled or invalid; previous directory restored",
                                    title: "Previous directory restored",
                                    "×"
                                }
                            },
                        }
                    }

                    Button { r#type: "submit", disabled: is_loading || is_saving(), "Save" }
                }
            }
        }
    }
}