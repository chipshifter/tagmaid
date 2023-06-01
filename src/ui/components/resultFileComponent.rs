use dioxus::prelude::*;
use crate::TagFile;

pub fn render(cx: Scope) -> Element {
    cx.render(rsx! {
        style { include_str!("css/resultFileComponent.css") }
        div {
            span { "thing" }
            h1 { "h1" }
        }
    })
}
