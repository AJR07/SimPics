use crate::components::button::{Button, ButtonSize, ButtonVariant};
use dioxus_icons::lucide::{House, Settings};

use dioxus::prelude::*;
#[css_module("/src/components/navbar/style.css")]
struct Styles;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Pages {
    Home,
    Settings,
}

#[derive(PartialEq, Clone, Props)]
pub struct NavbarProps {
    active_page: Signal<Pages>,
}

#[component]
pub fn Navbar(mut props: NavbarProps) -> Element {
    rsx! {
        nav { class: Styles::navbar,
            Button {
                variant: ButtonVariant::Outline,
                size: ButtonSize::Icon,
                onclick: move |_| {
                    props.active_page.set(Pages::Home);
                },
                House { size: "16px" }
            }

            Button {
                variant: ButtonVariant::Outline,
                size: ButtonSize::Icon,
                onclick: move |_| {
                    props.active_page.set(Pages::Settings);
                },
                Settings { size: "16px" }
            }
        }
    }
}
