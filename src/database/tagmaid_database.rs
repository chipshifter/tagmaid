//! TagMaidDatabase is the high-level component for managing the database
//! You probably want to use this if you deal with the files one way or another.
//! It is built on top of Arc<> and therefore can be cloned cheaply.
//! It is initialised once in main(), so a full restart would be required to change it.
use crate::data::{cache::TagMaidCache, tag_file::TagFile};
use crate::database::tag_database::TagDatabase;
use anyhow::{Context, Result};
use log::*;
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};

pub struct TagMaidDatabase {
    pub filesystem_db: Arc<Mutex<TagDatabase>>,
    cache: Arc<TagMaidCache>,
}

impl Clone for TagMaidDatabase {
    fn clone(&self) -> Self {
        return TagMaidDatabase {
            filesystem_db: Arc::clone(&self.filesystem_db),
            cache: Arc::clone(&self.cache),
        };
    }
}

/// Initialises the database.
pub fn init() -> TagMaidDatabase {
    // TODO: Put db_name in Config
    let db_name = "frank";
    let filesystem_db: TagDatabase = TagDatabase::initialise(db_name.to_owned(), None).unwrap();
    info!("Initialising TagMaidDatabse of name {db_name}");
    return TagMaidDatabase {
        filesystem_db: Arc::new(Mutex::new(filesystem_db)),
        cache: Arc::new(TagMaidCache::init()),
    };
}

impl TagMaidDatabase {
    pub fn get_fs_db(&self) -> Arc<Mutex<TagDatabase>> {
        return self.filesystem_db.clone();
    }

    pub fn get_cache(&self) -> Arc<TagMaidCache> {
        return self.cache.clone();
    }

    pub fn update_tagfile(&self, tf: &TagFile) -> Result<()> {
        info!("Updating {tf}");

        // Clearing search cache; since a file has been
        // modified this could affect searches, so the
        // old caches have to be invalidated
        // We also need to update the TagFile cache because the TagFile
        // has a `tags` attribute that is used and can be changed

        match self.get_cache().clear_results_cache() {
            Ok(_ok) => {
                info!("Clearing search cache because of edit.");
            }
            Err(_err) => {}
        }

        let fs_db_mutex = &self.get_fs_db();
        let fs_db = fs_db_mutex.lock().unwrap();
        let sql_db = &fs_db.sqlite_database;

        // Sqlite
        if sql_db.get_tagfile_from_hash(&tf.file_hash).is_err() {
            // File isn't in db
            info!("Updating {tf}: File not present in SQL database, uploading it");

            // Filesystem
            // Uploads file if it doesn't exist
            let uploaded_file = fs_db.upload_file(tf)?;

            sql_db.add_file(&uploaded_file)?;
        } else {
            if (&tf.tags).is_empty() {
                // File is already in database AND has no tags; we delete
                info!("Updating {tf}: File has no tags, removing it");
                sql_db.remove_file(&tf)?;

                // Removing the TagFile from the cache
                match self.get_cache().clear_tagfile_cache(tf.clone()) {
                    Ok(_ok) => {
                        info!("Clearing TagFile cache for {tf}.");
                    }
                    Err(_err) => {}
                }

                return Ok(());
            }
        }

        match self.get_cache().cache_tagfile(tf.clone()) {
            Ok(_ok) => {
                info!("Updating TagFile cache for {tf}.");
            }
            Err(_err) => {}
        }

        info!("Updating {tf}: Updating tags to SQL");
        sql_db.update_tags_to_file(tf)?;

        Ok(())
    }

    pub fn get_tagfile_from_hash(&self, hash: &Vec<u8>) -> Result<TagFile> {
        debug!(
            "Getting TagFile from file hash {} (trimmed)",
            crate::data::tag_util::trimmed_hash_hex(&hash)?
        );

        // Attempts to access cache and retrieve TagFile if it is there.
        match self.get_cache().get_tagfile(hash) {
            Some(cached_tagfile) => {
                debug!("Retrieved TagFile cache");
                return Ok(cached_tagfile.clone());
            }
            None => {}
        }

        // It wasn't cached, so we perform a SQL search
        let fs_db_mutex = &self.get_fs_db();
        let fs_db = fs_db_mutex.lock().unwrap();
        let sql_db = &fs_db.sqlite_database;
        let tagfile = sql_db.get_tagfile_from_hash(&hash)?;

        // This part is for caching the TagFile because it wasn't in the cache when
        // we checked.
        match self.get_cache().cache_tagfile(tagfile.clone()) {
            Ok(()) => {}
            Err(err) => {
                info!("WARNING: get_tagfile_from_hash(): Couldn't open cache as mutable because it was already being borrowed: {err}");
            }
        }

        return Ok(tagfile);
    }

    pub fn get_tags_from_hash(&self, hash: &Vec<u8>) -> Result<HashSet<String>> {
        debug!(
            "Getting tags from file hash {} (trimmed)",
            crate::data::tag_util::trimmed_hash_hex(&hash)?
        );
        let tags = &self.get_tagfile_from_hash(&hash)?.tags;
        return Ok(tags.to_owned());
    }

    pub fn get_tag_count(&self, tag: &str) -> Option<i64> {
        let fs_db_mutex = &self.get_fs_db();
        let fs_db = fs_db_mutex.lock().unwrap();
        let sql_db = &fs_db.sqlite_database;
        return sql_db.get_tag_count(tag).ok();
    }

    #[cfg(test)]
    fn create_random_tagmaiddatabase() -> TagMaidDatabase {
        return TagMaidDatabase {
            filesystem_db: Arc::new(Mutex::new(TagDatabase::create_random_tagdatabase())),
            cache: Arc::new(TagMaidCache::init()),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_tagfile_upload_and_retrieve() {
        let tf = TagFile::create_random_tagfile();
        let tf_hash = tf.file_hash.clone();

        let db = TagMaidDatabase::create_random_tagmaiddatabase();
        // Nothing was uploaded yet so it shouldn't find any TagFile
        assert!(db.get_tagfile_from_hash(&tf_hash).is_err());

        // We upload `tf`
        assert!(db.update_tagfile(&tf).is_ok());

        // We check if it found the right TagFile
        assert_eq!(db.get_tagfile_from_hash(&tf_hash).ok(), Some(tf));
    }

    
    fn
}
