pub mod tabs;

use crate::{TagFile, TagMaidDatabase, UITagmaidDatabase};
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_router::{Link, Redirect, Route, Router};

pub fn render(cx: Scope) -> Element {
    cx.render(rsx! {
        Router {
            nav {
                class: "tabs",
                tabs::render {}
            }
            Route { to: "/", Redirect { to: "/search" } },
            Route { to: "/search", tabs::search_tab::render {} },
            Route { to: "/results", tabs::results_tab::render {} },
            Route { to: "/add", tabs::add_file_tab::render {} },
            Route { to: "/settings", tabs::settings_tab::render {} },
        }
    })
}

/// Acquires a shared state of the TagMaidDatabase instance initialised in main
/// Note that you don't need to use write() as there is only one TagMaidDatabase instance
/// which can be modified internally
fn get_tagmaiddatabase(cx: &ScopeState) -> Option<TagMaidDatabase> {
    match use_shared_state::<UITagmaidDatabase>(cx) {
        // TagMaidDatabase can be cloned cheaply
        Some(db) => Some(db.read().0.clone()),
        None => None,
    }
}
