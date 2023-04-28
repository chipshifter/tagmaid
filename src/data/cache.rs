//! Handmade cache
use crate::data::{search_command::Search, tag_info::TagInfo};
use crate::ui::TextureLabel;
use crate::TagFile;
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct TagMaidCache {
    // Caches the hash of a TagFile to the associated TagFile object (if it exists)
    tagfile_cache: RwLock<HashMap<Vec<u8>, TagFile>>,
    // Tag Info
    tag_cache: RwLock<HashMap<String, TagInfo>>,
    results_cache: RwLock<HashMap<Search, Vec<Vec<u8>>>>,
    thumbnails_cache: RwLock<HashMap<TextureLabel, egui::TextureHandle>>,
}

impl TagMaidCache {
    pub fn init() -> TagMaidCache {
        // TODO: Save/load cache from a file
        return TagMaidCache {
            tagfile_cache: RwLock::new(HashMap::new()),
            tag_cache: RwLock::new(HashMap::new()),
            results_cache: RwLock::new(HashMap::new()),
            thumbnails_cache: RwLock::new(HashMap::new()),
        };
    }

    // TagFile cache

    /// Caches a tagfile in a `HashMap` whose key is the tagfile's hash and the value is the tagfile itself.
    pub fn cache_tagfile(&self, tf: TagFile) -> Result<()> {
        match self.tagfile_cache.try_write() {
            Ok(mut cache) => {
                cache.insert(tf.file_hash.clone(), tf);
                return Ok(());
            }
            Err(err) => bail!("Couldn't write to TagFile cache to cache file: {err}"),
        }
    }

    /// Removes a TagFile from the cache
    pub fn clear_tagfile_cache(&self, tf: TagFile) -> Result<()> {
        match self.tagfile_cache.try_write() {
            Ok(mut cache) => {
                cache.remove(&tf.file_hash);
                return Ok(());
            }
            Err(err) => bail!("Couldn't write to TagFile cache to cache file: {err}"),
        }
    }

    /// Takes a tagfile hash in the argument. Returns `Some(tagfile)`.
    /// Returns `None` if tagfile wasn't cached.
    pub fn get_tagfile(&self, hash: &Vec<u8>) -> Option<TagFile> {
        match self.tagfile_cache.try_read() {
            Ok(cache) => cache.get(hash).cloned(),
            Err(_err) => None,
        }
    }

    // Search cache

    /// Clears the entire search result cache. This is done is a user adds a new file or edits one
    /// (in which case searches need to be updated)
    pub fn clear_results_cache(&self) -> Result<()> {
        match self.results_cache.try_write() {
            Ok(mut cache) => {
                cache.clear();
                return Ok(());
            }
            Err(err) => bail!("Couldn't write to search cache to clear it: {err}"),
        }
    }

    /// Caches a search query (of type `crate::data::search_command::Search`) and its results
    /// in a HashMap
    pub fn cache_search(&self, search_query: Search, result_hashes: Vec<Vec<u8>>) -> Result<()> {
        match self.results_cache.try_write() {
            Ok(mut cache) => {
                cache.insert(search_query, result_hashes);
                return Ok(());
            }
            Err(err) => bail!("Couldn't write to TagFile cache to cache search: {err}"),
        }
    }

    /// Retrieves cached search. Returns `Some(Vec<Vec<u8>>)` if found, `None` otherwise.
    pub fn get_search(&self, search_query: &Search) -> Option<Vec<Vec<u8>>> {
        match self.results_cache.try_read() {
            Ok(cache) => cache.get(search_query).cloned(),
            Err(_err) => None,
        }
    }

    // Thumbnails cache

    pub fn cache_thumbnail(&self, label: TextureLabel, texture: egui::TextureHandle) -> Result<()> {
        match self.thumbnails_cache.try_write() {
            Ok(mut cache) => {
                cache.insert(label, texture);
                return Ok(());
            }
            Err(err) => {
                bail!("Couldn't write to thumbnail cache to cache thumbnail texture: {err}");
            }
        }
    }

    pub fn get_thumbnail(&self, label: &TextureLabel) -> Option<egui::TextureHandle> {
        match self.thumbnails_cache.try_read() {
            Ok(cache) => cache.get(label).cloned(),
            Err(_err) => None,
        }
    }

    // Tag Cache

    pub fn cache_tag_info(&self, tag_info: TagInfo) -> Result<()> {
        match self.tag_cache.try_write() {
            Ok(mut cache) => {
                cache.insert(tag_info.get_tag(), tag_info);
                return Ok(());
            }
            Err(err) => {
                bail!("Couldn't write to tag cache: {err}");
            }
        }
    }

    pub fn get_tag_info(&self, tag: &str) -> Option<TagInfo> {
        match self.tag_cache.try_read() {
            Ok(cache) => cache.get(tag).cloned(),
            Err(_err) => None,
        }
    }
}
