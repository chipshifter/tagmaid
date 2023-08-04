use std::collections::HashSet;

use crate::{data::search_command::Search, database::tagmaid_database::TagMaidDatabase, TAGMAID_DATABASE};
use anyhow::{Context, Result};

pub fn do_search(query: &str) -> Result<Vec<Vec<u8>>> {
    let query = Search::from_string(query).context("Couldn't parse search query text")?;
    let db = &TAGMAID_DATABASE;

    // v : search query vector
    let mut cands = match query.first_tag() {
        Some(first_tag) => db.get_hashes_from_tag(&first_tag).unwrap_or(HashSet::new()),
        None => db.get_all_file_hashes()?,
    };

    cands.retain(|hash| {
        let tags = &db.get_tags_from_hash(hash);
        match tags {
            Ok(tags) => query.filter_post(&tags),
            Err(..) => false,
        }
    });

    let results_vec: Vec<Vec<u8>> = cands.into_iter().collect();

    Ok(results_vec)
}
