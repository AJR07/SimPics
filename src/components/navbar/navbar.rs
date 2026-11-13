use crate::components::navbar::{NavButton, NavSection};
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
pub fn Navbar(props: NavbarProps) -> Element {
    rsx! {
        nav { class: Styles::navbar,
            NavSection { title: "Workspace",
                NavButton {
                    text: "Home",
                    icon: House,
                    linked_page: Pages::Home,
                    active_page: props.active_page.clone(),
                }

                NavButton {
                    text: "Settings",
                    icon: Settings,
                    linked_page: Pages::Settings,
                    active_page: props.active_page.clone(),
                }
            }
        }
    }
}
