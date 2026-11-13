use dioxus::prelude::*;

#[css_module("/src/components/navbar/style.css")]
struct Styles;

#[component]
pub fn NavSection(title: &'static str, children: Element) -> Element {
    rsx! {
		div { class: Styles::navsection,
			p { "{title.to_uppercase()}" }
			{children}
		}
	}
}
