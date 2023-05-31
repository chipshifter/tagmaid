use dioxus::prelude::*;

pub fn render(cx: Scope) -> Element {
    cx.render(rsx! {
        h1 { "Settings" }
    })
}
