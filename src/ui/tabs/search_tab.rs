use crate::data::search_command::Search;
use crate::get_ui_data;
use crate::TagMaidDatabase;
use crate::UIData;

use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_router::{Redirect, Router};
use std::collections::HashSet;

use anyhow::{bail, Context, Result};

pub fn render(cx: Scope) -> Element {
    let draft = use_ref(cx, String::new);
    let redirect = use_ref(cx, || false);

    let do_the_search = move |searching: bool| {
        if searching == true {
            let ui_data = get_ui_data(cx);
            let results_vec = do_search(&draft.read(), ui_data.read().db()).ok();
            match results_vec {
                Some(results) => {
                    ui_data.write().update_search_results(results);
                }
                None => {}
            }
            // Redirect to results
            redirect.set(true);
        }
    };

    cx.render(rsx! {
        h1 { "Search" }
        input {
            autofocus: "true",
            value: "{draft.read()}",
            oninput: move |event| draft.set(event.value.clone()),
            onkeypress: move |event| {
                if event.key() == Key::Enter && !draft.read().is_empty() {
                    draft.set(String::new());
                    do_the_search(true);
                }
            }
        }
        button {
            onclick: move |_| do_the_search(true),
            "Search"
        }

        redirect.read().clone().then(|| {
            rsx!( Redirect { to: "/results" } )
        })
    })
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
