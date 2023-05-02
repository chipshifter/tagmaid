//! TagMaidDatabase is the high-level component for managing the database
//! You probably want to use this if you deal with the files one way or another.
//! It is built on top of Arc<> and therefore can be cloned cheaply.
//! It is initialised once in main(), so a full restart would be required to change it.
use crate::data::{cache::TagMaidCache, tag_file::TagFile, tag_info::TagInfo};
use crate::database::{
    fs_database::FsDatabase, sqlite_database::SqliteDatabase, sqlite_files::FilesDatabase,
    sqlite_tags::TagsDatabase, sqlite_taginfo::TagInfoDatabase
};
use anyhow::{Context, Result};
use log::*;
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

pub struct TagMaidDatabase {
    pub filesystem_db: Arc<Mutex<FsDatabase>>,
    sqlite_db: Arc<Mutex<SqliteDatabase>>,
    cache: Arc<TagMaidCache>,
}

impl Clone for TagMaidDatabase {
    fn clone(&self) -> Self {
        return TagMaidDatabase {
            filesystem_db: Arc::clone(&self.filesystem_db),
            sqlite_db: Arc::clone(&self.sqlite_db),
            cache: Arc::clone(&self.cache),
        };
    }
}

pub fn get_database_path(custom_parent_path: Option<PathBuf>) -> Result<PathBuf> {
    // TODO: Custom path in a config file?
    match custom_parent_path {
        Some(path) => {
            let mut tagmaid_path = path.clone();
            tagmaid_path.push("tag-maid");
            Ok(tagmaid_path)
        }
        None => {
            let mut local_path = dirs::data_local_dir()
                .context("Database: Couldn't find local user data path for storing the database")?;
            local_path.push("tag-maid");
            Ok(local_path)
        }
    }
}

/// Initialises the database.
pub fn init() -> TagMaidDatabase {
    // TODO: Put db_name in Config
    let db_name = "frank";

    // None: no custom path, use local (we will change that to deal with configs in the future)
    let mut db_path = get_database_path(None).unwrap();

    // Create /home/user/.local/.../tag-maid folder otherwise everything breaks
    if !std::path::Path::new(&db_path).exists() {
        std::fs::create_dir(&db_path)
            .context(format!(
                "Can't create '{}' folder because it already exists",
                &db_path.display()
            ))
            .ok();
    }

    db_path.push(db_name);

    let filesystem_db: FsDatabase = FsDatabase::initialise(&db_path).unwrap();
    let sqlite_db = SqliteDatabase::initialise(&db_path).unwrap();
    info!("Initialising TagMaidDatabse of name {db_name}");
    return TagMaidDatabase {
        filesystem_db: Arc::new(Mutex::new(filesystem_db)),
        sqlite_db: Arc::new(Mutex::new(sqlite_db)),
        cache: Arc::new(TagMaidCache::init()),
    };
}

impl TagMaidDatabase {
    pub fn get_fs_db(&self) -> Arc<Mutex<FsDatabase>> {
        return self.filesystem_db.clone();
    }

    pub fn get_sql_db(&self) -> Arc<Mutex<SqliteDatabase>> {
        return self.sqlite_db.clone();
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

        let sql_db_mutex = &self.get_sql_db();
        let sql_db = sql_db_mutex.lock().unwrap();

        // Sqlite
        if FilesDatabase::get_tagfile_from_hash(sql_db.get_connection(), &tf.file_hash).is_err() {
            // File isn't in db
            info!("Updating {tf}: File not present in SQL database, uploading it");

            // Filesystem
            // Uploads file if it doesn't exist
            let fs_db_mutex = &self.get_fs_db();
            let uploaded_file = fs_db_mutex.lock().unwrap().upload_file(tf)?;

            FilesDatabase::add_file(sql_db.get_connection(), &uploaded_file)?;
        } else {
            if (&tf.tags).is_empty() {
                // File is already in database AND has no tags; we delete
                info!("Updating {tf}: File has no tags, removing it");
                FilesDatabase::remove_file(sql_db.get_connection(), &tf)?;

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

        info!("Updating {tf}: Updating tags at FilesDatabase");
        FilesDatabase::update_tags_to_file(sql_db.get_connection(), tf)?;
        info!("Updating {tf}: Updating tags at TagsDatabase");
        TagsDatabase::add_tags(sql_db.get_connection(), tf.get_tags())?;
        Ok(())
    }

    pub fn remove_tagfile(&self, file: &TagFile) -> Result<()> {
        let sql_db_mutex = &self.get_sql_db();
        let sql_db = sql_db_mutex.lock().unwrap();
        let sql_db_connection = sql_db.get_connection();
        for tag in file.get_tags() {
            TagsDatabase::decrease_tag_count_by_one(sql_db_connection, tag)?;
        }
        FilesDatabase::remove_file(sql_db_connection, file)?;
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
        let sql_db_mutex = &self.get_sql_db();
        let sql_db = sql_db_mutex.lock().unwrap();
        let tagfile = FilesDatabase::get_tagfile_from_hash(sql_db.get_connection(), &hash)?;

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
        let sql_db_mutex = &self.get_sql_db();
        let sql_db = sql_db_mutex.lock().unwrap();
        let tags = FilesDatabase::get_tagfile_from_hash(sql_db.get_connection(), &hash)?.tags;
        return Ok(tags.to_owned());
    }

    // TagInfo

    pub fn update_tag_info(&self, tag_info: &TagInfo) -> Result<()> {
        let sql_db_mutex = &self.get_sql_db();
        let sql_db = sql_db_mutex.lock().unwrap();

        TagInfoDatabase::update_tag_info(sql_db.get_connection(), tag_info)?;
        Ok(())
    }

    pub fn get_tag_info(&self, tag: String) -> Option<TagInfo> {
        let sql_db_mutex = &self.get_sql_db();
        let sql_db = sql_db_mutex.lock().unwrap();
    
        match self.get_cache().get_tag_info(&tag) {
            Some(tag_info) => Some(tag_info),
            None => {
                let tag_info =
                    TagInfoDatabase::get_tag_info_from_tag(sql_db.get_connection(), &tag).ok();

                if tag_info.is_some() {
                    self.get_cache().cache_tag_info(tag_info.clone().unwrap()).ok();
                }

                return tag_info;
            }
        }
    }

    pub fn get_tag_count(&self, tag: &str) -> Option<i64> {
        let sql_db_mutex = &self.get_sql_db();
        let sql_db = sql_db_mutex.lock().unwrap();
        return TagsDatabase::get_tag_count(sql_db.get_connection(), tag).ok();
    }

    pub fn get_hashes_from_tag(&self, tag: &str) -> Result<HashSet<Vec<u8>>> {
        let sql_db_mutex = &self.get_sql_db();
        let sql_db = sql_db_mutex.lock().unwrap();

        let hashes = FilesDatabase::get_hashes_from_tag(sql_db.get_connection(), &tag)
            .with_context(|| format!("Database: Couldn't get hashes from tag {}", &tag))?;
        Ok(hashes)
    }

    pub fn get_all_file_hashes(&self) -> Result<HashSet<Vec<u8>>> {
        let sql_db_mutex = &self.get_sql_db();
        let sql_db = sql_db_mutex.lock().unwrap();

        let hashes = FilesDatabase::get_all_file_hashes(sql_db.get_connection())
            .context("Database: Couldn't get all file hashes")?;
        Ok(hashes)
    }

    #[cfg(test)]
    pub fn create_random_tagmaiddatabase() -> TagMaidDatabase {
        let random_fs_db = FsDatabase::create_random_fsdatabase();
        let random_fs_db_path = random_fs_db.path.clone();
        return TagMaidDatabase {
            filesystem_db: Arc::new(Mutex::new(random_fs_db)),
            sqlite_db: Arc::new(Mutex::new(
                SqliteDatabase::initialise(&random_fs_db_path).unwrap(),
            )),
            cache: Arc::new(TagMaidCache::init()),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_tagmaiddatabase_structure_be_correct() {
        let db = TagMaidDatabase::create_random_tagmaiddatabase();
        // (sigh)
        let fs_db = Arc::try_unwrap(db.filesystem_db)
            .unwrap()
            .into_inner()
            .unwrap();

        // The database when initialised creates the following things in the parent folder:
        //  - sqlite.db for the SQLite database
        //  - A "files" folder that contains all the uploaded files

        // fs_db.contents is a ReadDir reading from the database path.
        // We turn that into a vector of PathBufs to compare with what is expected
        let path_iter: Vec<PathBuf> = fs_db.contents.map(|f| f.unwrap().path()).collect();

        // "files"
        let mut files_path = fs_db.path.clone();
        files_path.push("files");

        // "sqlite.db"
        let mut sqlite_db_file_path = fs_db.path.clone();
        sqlite_db_file_path.push("sqlite.db");

        // Check if files in db.contents
        assert_eq!(path_iter, vec![files_path, sqlite_db_file_path]);
    }

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

    #[test]
    fn should_tag_counts_update() {
        let db = TagMaidDatabase::create_random_tagmaiddatabase();

        let mut tf_1 = TagFile::create_random_tagfile();
        let _ = tf_1.add_tag("test_tag");
        let _ = tf_1.add_tag("another_tag");
        assert!(db.update_tagfile(&tf_1).is_ok());

        let mut tf_2 = TagFile::create_random_tagfile();
        let _ = tf_2.add_tag("test_tag");
        assert!(db.update_tagfile(&tf_2).is_ok());

        assert_eq!(db.get_tag_count("test_tag"), Some(2));
        assert_eq!(db.get_tag_count("another_tag"), Some(1));

        assert!(db.remove_tagfile(&tf_1).is_ok());

        assert_eq!(db.get_tag_count("test_tag"), Some(1));
        assert_eq!(db.get_tag_count("another_tag"), None);

        assert!(db.update_tagfile(&tf_1).is_ok());

        assert_eq!(db.get_tag_count("test_tag"), Some(2));
        assert_eq!(db.get_tag_count("another_tag"), Some(1));

        assert!(db.remove_tagfile(&tf_2).is_ok());

        assert_eq!(db.get_tag_count("test_tag"), Some(1));
        assert_eq!(db.get_tag_count("another_tag"), Some(1));
    }

    #[test]
    fn should_get_tag_info() {
        let db = TagMaidDatabase::create_random_tagmaiddatabase();

        let mut tf = TagFile::create_random_tagfile();
        let _ = tf.add_tag("test_tag");
        let _ = tf.add_tag("another_tag");

        assert!(db.update_tagfile(&tf).is_ok());

        assert_eq!(db.get_tag_info("test_tag".to_string()), Some(TagInfo {
            tag: "test_tag".to_string(),
            upload_count: 1
        }));

        assert_eq!(db.get_tag_info("another_tag".to_string()), Some(TagInfo {
            tag: "another_tag".to_string(),
            upload_count: 1
        }));

        assert_eq!(db.get_tag_info("non_existant_tag".to_string()), None);
    }
}
