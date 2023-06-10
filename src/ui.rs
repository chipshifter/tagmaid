pub mod tabs;

use crate::{TagFile, TagMaidDatabase};
use dioxus::{prelude::*, html::input_data::keyboard_types::Key};
use dioxus_router::{Redirect, Route, Router};

pub fn render(cx: Scope) -> Element {
    cx.render(rsx! {
        Router {
            nav { 
                class: "tabs",
                crate::ui::tabs::render {}
            }
            main {
                Route { to: "/search", crate::ui::tabs::search_tab::render {} }
                Route { to: "/results", crate::ui::tabs::results_tab::render {} }
                Route { to: "/add", crate::ui::tabs::add_file_tab::render {} }
                Route { to: "/settings", crate::ui::tabs::settings_tab::render {} }

                Redirect { to: "/search" }
            }
        }
    })
}
