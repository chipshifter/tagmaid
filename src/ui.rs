pub mod tabs;

use crate::{TagFile, TagMaidDatabase};
use dioxus::prelude::*;
use dioxus_router::{Redirect, Route, Router};

pub fn render(cx: Scope) -> Element {
    cx.render(rsx! {
        Router {
            header {
                crate::ui::tabs::render {}
            }
            body {
                Route { to: "/search", crate::ui::tabs::search_tab::render {} }
                Route { to: "/results", crate::ui::tabs::results_tab::render {} }
                Route { to: "/add", crate::ui::tabs::add_file_tab::render {} }
                Route { to: "/settings", crate::ui::tabs::settings_tab::render {} }

                Redirect { to: "/search" }
            }
        }
    })
}
