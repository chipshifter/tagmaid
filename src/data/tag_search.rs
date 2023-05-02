//! Utility functions for searching tags. The search is not for files
//! (which searches in the `_files` database table) but for the tags
//! themselves (stored in the `_tags` table). This is currently used for search suggestions.
use std::collections::BTreeMap;
use anyhow::Result;
use rusqlite::Connection;
use crate::data::{tag_info::TagInfo};
use crate::database::sqlite_tags::TagsDatabase;
pub struct TagSearch;

impl TagSearch {
    /// Searches in _tags for tags that start with the given string.
    /// For instance: you have the tags brie, brioche, branch in your database
    /// stored somewhere in tags. Then, `get_tags_starting_with(&db, "bri")` will return
    /// a BTreeMap of the `brie` and `brioche` tags because they started with `bri`, with their
    /// upload count as well.
    pub fn get_tags_starting_with(db: &Connection, string: &str) -> Result<BTreeMap<i64, Vec<String>>> {
        let mut quer = db.prepare(
            format!("SELECT tag_name, upload_count FROM _tags WHERE tag_name LIKE \"{string}%\"").as_str(),
        )?;
        let search_result = quer.query_map([], |row| Ok(TagInfo {tag: row.get(0)?, upload_count: row.get(1)?}))?;
        let mut tag_infos: BTreeMap<i64, Vec<String>> = BTreeMap::new();
        for result in search_result {
            let tag_info = result?;
            let upload_count = tag_info.get_upload_count();
            let tag = tag_info.get_tag();

            let mut tags_vec: Vec<String> = Vec::new();

            if let Some(vec) = tag_infos.get_mut(&upload_count) {
                tags_vec = vec.drain(..).collect();
            }

            tags_vec.push(tag);

            tag_infos.insert(upload_count, tags_vec);
        }
        Ok(tag_infos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::sqlite_database::SqliteDatabase;

    #[test]
    fn should_empty_tag_search_work() {
        let sql_db = SqliteDatabase::get_random_db_connection();
        let db = sql_db.get_connection();

        let search_result_map = TagSearch::get_tags_starting_with(&db, "").unwrap();

        // The result should only have "brie" (with upload_count 12) and "brioche", NOT "branch"
        let upload_count_keys: Vec<_> = search_result_map.keys().cloned().collect();
        assert!(upload_count_keys.is_empty());

        let upload_count_values: Vec<_> = search_result_map.values().cloned().collect();
        assert!(upload_count_values.is_empty());
    }

    #[test]
    fn should_tag_search_and_filter() {
        let sql_db = SqliteDatabase::get_random_db_connection();
        let db = sql_db.get_connection();

        assert!(TagsDatabase::add_tag(db, "brie").is_ok());
        assert!(TagsDatabase::set_tag_count(db, "brie", 12).is_ok());

        assert!(TagsDatabase::add_tag(db, "brioche").is_ok());
        assert!(TagsDatabase::set_tag_count(db, "brioche", 34).is_ok());

        assert!(TagsDatabase::add_tag(db, "branch").is_ok());
        assert!(TagsDatabase::set_tag_count(db, "branch", 56).is_ok());

        let search_result_map = TagSearch::get_tags_starting_with(&db, "bri").unwrap();

        // The result should only have "brie" (with upload_count 12) and "brioche", NOT "branch"
        let upload_count_keys: Vec<_> = search_result_map.keys().cloned().collect();
        assert_eq!(upload_count_keys, [12, 34]);

        let upload_count_values: Vec<_> = search_result_map.values().cloned().collect();
        assert_eq!(upload_count_values, [["brie"], ["brioche"]]);

        // No "branch"
        assert!(!search_result_map.contains_key(&56));
    }

    #[test]
    fn should_colliding_tag_search_work() {
        let sql_db = SqliteDatabase::get_random_db_connection();
        let db = sql_db.get_connection();

        // If two tags in the search have the same upload count
        // they will have the same key value
        // So the value of that key will be the two tags
        assert!(TagsDatabase::add_tag(db, "brie").is_ok());
        assert!(TagsDatabase::set_tag_count(db, "brie", 12).is_ok());

        assert!(TagsDatabase::add_tag(db, "brioche").is_ok());
        assert!(TagsDatabase::set_tag_count(db, "brioche", 12).is_ok());

        let search_result_map = TagSearch::get_tags_starting_with(&db, "bri").unwrap();

        // The result should only have "brie" (with upload_count 12) and "brioche", NOT "branch"
        let upload_count_keys: Vec<_> = search_result_map.keys().cloned().collect();
        assert_eq!(upload_count_keys, [12]);

        let upload_count_values: Vec<_> = search_result_map.values().cloned().collect();
        assert_eq!(upload_count_values, [["brie", "brioche"]]); // Both tags in the same Vec
    }
}
