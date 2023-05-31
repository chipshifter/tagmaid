pub mod add_file_tab;
pub mod results_tab;
pub mod search_tab;
pub mod settings_tab;

use dioxus::prelude::*;
use dioxus_router::Link;

pub fn render(cx: Scope) -> Element {
    cx.render(rsx! {
        table { class: "tabs",
            td {
                Link { to: "/search", "Search" }
            }
            td {
                Link { to: "/results", "Results" }
            }
            td {
                Link { to: "/add", "Add" }
            }
            td {
                Link { to: "/settings", "Settings" }
            }
        }
        hr {}
    })
}