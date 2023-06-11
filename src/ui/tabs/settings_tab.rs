use dioxus::prelude::*;

pub fn render(cx: Scope) -> Element {
    println!("hey");
    
    cx.render(rsx! {
        h1 { "Settings" }
    })
}
