use crate::UIData;
use crate::TagMaidDatabase;
use crate::data::search_command::Search;

use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use std::collections::HashSet;

use anyhow::{bail, Context, Result};

pub fn render<'a>(cx: &'a ScopeState) -> Element<'a> {
    let ui_data = use_shared_state::<UIData>(cx).unwrap();
    let draft = use_ref(cx, String::new);

    cx.render(rsx! {
        h1 { "Search" }
        input {
            autofocus: "true",
            value: "{draft.read()}",
            oninput: move |event| draft.set(event.value.clone()),
            onkeydown: move |event| {
                if event.key() == Key::Enter && !draft.read().is_empty() {
                    // Do search
                    println!("Query entered: {}", &draft.read());
                    println!("all file hashes: {:?}", ui_data.read().db().get_all_file_hashes());
                }
            }
        }
        button {
            onclick: move |_| {
                // Do search
                let results_vec = do_search(&draft.read(), ui_data.read().db()).ok();
                match results_vec {
                    Some(results) => {
                        ui_data.write().search_results_hashes.clear();
                        // dumb
                        for result in results {
                            ui_data.write().search_results_hashes.push(result);
                        }
                    },
                    None => {
                        println!("Something bad happened idk");
                    }
                }
            },
            "click"
        }
        button {
            onclick: move |_| {
                // Do search
                println!("{:?}", ui_data.read().search_results_hashes);
            },
            "print uidata"
        }
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
            bail!("NO")
        }
    }
    // match Search::from_string(&self.search) {
    //     Ok(v) => {
    //         *self.update_search.lock().unwrap() = true;
    //         self.search_err = None;
    //         let nbool = Arc::clone(&self.update_search);
    //         let nres = Arc::clone(&self.results);
    //         let db = self.db.clone();
    //         let search = v.clone();
    //         let thumbnail_paths = self.thumbnail_paths.clone();
    //         let mut is_cached = false;
    //         self.search_options = Some(v.clone());

    //         // Try finding a cached search

    //         match self.db.get_cache().get_search(&search) {
    //             Some(search_results) => {
    //                 is_cached = true;
    //                 *nres.lock().unwrap() = search_results.clone();
    //                 *nbool.clone().lock().unwrap() = false;
    //             }
    //             None => {}
    //         }

    //         // Search wasn't cached
    //         if !is_cached {
    //             self.search_thread = Some(std::thread::spawn(move || {
    //                 match Self::get_results(
    //                     search,
    //                     nres.clone(),
    //                     nbool,
    //                     db,
    //                     thumbnail_paths,
    //                 ) {
    //                     Ok(..) => {}
    //                     Err(..) => {
    //                         nres.clone().lock().unwrap().clear();
    //                     }
    //                 }
    //             }));
    //         }
    //         self.query.replace(v);

    //         // Search is done, send user to results page
    //         self.mode = ViewPage::Results;
    //     }
    //     Err(s) => {
    //         self.search_err = Some(s.to_string());
    //     }
    // }
}
