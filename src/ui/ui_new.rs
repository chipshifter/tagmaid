use crate::{TagFile, TagMaidDatabase};
use dioxus::prelude::*;
use dioxus_router::{Redirect, Route, Router};

#[derive(Copy, Clone)]
pub struct UIData {
    pub db: &'static TagMaidDatabase,
}

impl UIData {
    pub fn new(db: &'static TagMaidDatabase) -> Self {
        Self { db: db }
    }

    pub fn db(&self) -> TagMaidDatabase {
        self.db.clone()
    }
}

pub fn render<'a>(cx: &'a ScopeState, ui_data: &'a UseState<UIData>) -> Element<'a> {
    cx.render(rsx! {
        Router {
            header {
                crate::ui::tabs::render {}
            }
            Route { to: "/search", crate::ui::tabs::search_tab::render(cx, ui_data.get()) {} }
            Route { to: "/results", crate::ui::tabs::results_tab::render(cx, ui_data.get()) {} }
            Route { to: "/add", crate::ui::tabs::add_file_tab::render {} }
            Route { to: "/settings", crate::ui::tabs::settings_tab::render {} }

            Redirect { from: "", to: "/search" }
        }
    })
}
