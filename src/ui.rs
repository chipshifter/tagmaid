pub mod tabs;

use std::borrow::BorrowMut;

use crate::{TagFile, TagMaidDatabase, UITagmaidDatabase};
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

/// Acquires a shared state of the TagMaidDatabase instance initialised in main
/// Note that you don't need to use write() as there is only one TagMaidDatabase instance
/// which can be modified internally
fn get_tagmaiddatabase(cx: &ScopeState) -> Option<TagMaidDatabase> {
    match use_shared_state::<UITagmaidDatabase>(cx) {
        // TagMaidDatabase can be cloned cheaply
        Some(db) => Some(db.read().0.clone()),
        None => None
    }
}
