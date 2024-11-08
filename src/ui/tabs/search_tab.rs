use crate::data::search_command::Search;
use crate::get_ui_data;
use crate::ui::Route;
use crate::TagMaidDatabase;
use crate::UIData;

use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_router::prelude::navigator;
use std::collections::HashSet;

use anyhow::{bail, Context, Result};

pub fn render() -> Element {
    let mut draft = use_signal(String::new);
    let navigator = navigator();

    let do_the_search = move || {
        let mut ui_data = get_ui_data();
        let results_vec = do_search(&draft.read(), ui_data.db()).ok();
        match results_vec {
            Some(results) => {
                ui_data.update_search_results(results);
            }
            None => {}
        }
        // Redirect to results
        navigator.push(Route::Results {});
    };

    rsx! {
        h1 { "Search" }
        input {
            autofocus: "true",
            value: "{draft.read()}",
            oninput: move |event| draft.set(event.value()),
            onkeypress: move |event| {
                if event.key() == Key::Enter && !draft.read().is_empty() {
                    do_the_search()
                }
            }
        }
        button {
            onclick: move |_| do_the_search(),
            "Search"
        }
    }
}

pub fn do_search(query: &str, db: TagMaidDatabase) -> Result<Vec<Vec<u8>>> {
    match Search::from_string(query) {
        Ok(query_vector) => {
            // v : search query vector
            let mut cands = match query_vector.first_tag() {
                Some(first_tag) => db.get_hashes_from_tag(&first_tag).unwrap_or(HashSet::new()),
                None => db.get_all_file_hashes()?,
            };

            cands.retain(|hash| {
                let tags = &db.get_tags_from_hash(hash);
                match tags {
                    Ok(tags) => query_vector.filter_post(&tags),
                    Err(..) => false,
                }
            });

            let results_vec: Vec<Vec<u8>> = cands.into_iter().collect();

            Ok(results_vec)
        }
        Err(e) => {
            bail!(e)
        }
    }
}
