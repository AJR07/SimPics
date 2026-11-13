use dioxus::prelude::*;
use dioxus_icons::IconProps;

use crate::components::navbar::Pages;

#[css_module("/src/components/navbar/style.css")]
struct Styles;

#[derive(Clone, Props)]
pub struct NavButtonProps {
    text: &'static str,
    icon: Component<IconProps>,
    active_page: Signal<Pages>,
    linked_page: Pages,
}

// effectively ignore the icon variable for comparisons for now
impl PartialEq for NavButtonProps {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text &&
            self.active_page == other.active_page &&
            self.linked_page == other.linked_page
    }
}

#[component]
pub fn NavButton(mut props: NavButtonProps) -> Element {
    let Icon = props.icon;
    let selected = props.active_page.clone()() == props.linked_page.clone();

    // for styling
    let selected_class = if selected { Styles::navbutton_selected.to_string() } else { String::new() };
    rsx! {
        button {
            class: "{Styles::navbutton} {selected_class}",
            onclick: move |_| {
                props.active_page.set(props.linked_page);
            },
            Icon {
                size: "16px",
                class: if selected { Styles::navbutton_icon_selected },
            }
            p { "{props.text}" }
        }
    }
}
