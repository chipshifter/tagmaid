pub mod add_file_tab;
pub mod results_tab;
pub mod search_tab;
pub mod settings_tab;

use dioxus::prelude::*;
use dioxus_router::Link;

pub fn render(cx: Scope) -> Element {
    cx.render(rsx! {
        ol {
            li {
                class: "tab",
                Link { to: "/search", "Search" }
            }
            li {
                class: "tab",
                Link { to: "/results", "Results" }
            }
            li {
                class: "tab",
                Link { to: "/add", "Add" }
            }
            li {
                class: "tab",
                Link { to: "/settings", "Settings" }
            }
        }
        hr {}
    })
}
