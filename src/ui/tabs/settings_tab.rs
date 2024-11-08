use dioxus::prelude::*;

pub fn render() -> Element {
    println!("hey");
    
    rsx! {
        h1 { "Settings" }
    }
}
