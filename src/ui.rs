pub mod tabs;

use crate::{TagFile, TagMaidDatabase, UITagmaidDatabase};
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_router::prelude::*;

use tabs::search_tab::render as Search;
use tabs::results_tab::render as Results;
use tabs::add_file_tab::render as AddTab;
use tabs::settings_tab::render as Settings;

#[derive(Routable, PartialEq, Clone)]
enum Route {
    #[layout(NavBar)]
        #[route("/search")]
        #[redirect("/:.._seg", |_seg: Vec<String>| Route::Search {})]
        Search {},
        #[route("/results")]
        Results {},
        #[route("/add")]
        AddTab {},
        #[route("/settings")]
        Settings {},
    #[end_layout]
    #[route("/nothing")]
    Nothing {}
}

pub fn render() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn NavBar() -> Element {
    rsx!{
        nav {
            class: "tabs",
            tabs::render {}
        }
        Outlet::<Route> {}
    }
}

#[allow(non_snake_case)]
pub fn Nothing() -> Element {
    rsx!{}
}

/// Acquires a shared state of the TagMaidDatabase instance initialised in main
/// Note that you don't need to use write() as there is only one TagMaidDatabase instance
/// which can be modified internally
fn get_tagmaiddatabase() -> TagMaidDatabase {
    use_root_context::<UITagmaidDatabase>(|| panic!()).0
}
