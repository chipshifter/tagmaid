//! TagDatabase is the old database interface. It is in the process of being repurposed as the
//! "filesystem" interface, used for hardlinking files to the database path etc.
use crate::data::tag_file::TagFile;
use crate::database::sqlite_database::{SqliteDatabase, TagFileSqlite};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use log::*;
use rusqlite::Connection;
use std::collections::HashSet;
use std::fs::{self, File, ReadDir};
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::io::Write;
use rand::distributions::{Alphanumeric, DistString};

pub struct TagDatabase {
    pub name: String,
    pub path: PathBuf,
    pub contents: ReadDir,
    pub sqlite_database: SqliteDatabase,
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

fn hardlink_file_else_copy(old_path: &PathBuf, new_path: &PathBuf) -> Result<()> {
    info!(
        "hardlink_file_else_copy() - Hardlinking file from old path {} to new path {}",
        &old_path.display(),
        &new_path.display()
    );
    let hardlink_result = fs::hard_link(old_path, new_path).with_context(|| {
        format!(
            "Couldn't hardlink file from '{:?}' to '{:?}'",
            old_path, new_path
        )
    });
    if hardlink_result.is_err() {
        println!("nuuuuu dont fail hardlinkig");
        info!("hardlink_file_else_copy() - Hardlinking failed; COPYING instead.");
        fs::copy(old_path, new_path).with_context(|| {
            format!(
                "Couldn't copy file from '{:?}' to '{:?}'",
                old_path, new_path
            )
        })?;
    }
    Ok(())
}

impl TagDatabase {
    pub fn initialise(name: String, custom_path: Option<PathBuf>) -> Result<TagDatabase> {
        let mut path: PathBuf = get_database_path(custom_path.clone())?;

        if !Path::new(&path).exists() {
            fs::create_dir(&path).context(format!(
                "Can't create '{}' folder because it already exists",
                &path.display()
            ))?;
        }

        // IKA TODO: Handle edge cases for "name" variable
        path.push(&name);
        if !Path::new(&path).exists() {
            fs::create_dir(&path).context(format!(
                "Can't create '{}' folder because it already exists",
                &path.display()
            ))?;
        }

        let mut files_path = path.clone();
        files_path.push("files");
        if !Path::new(&files_path).exists() {
            fs::create_dir(&files_path).context(format!(
                "Can't create '{}' folder because it already exists",
                &files_path.display()
            ))?;
        }

        let sqlite_databases = SqliteDatabase::initialise(&name, custom_path)?;

        let db_folder = fs::read_dir(&path)?;
        Ok(TagDatabase {
            name: name,
            path: path,
            contents: db_folder,
            sqlite_database: sqlite_databases,
        })
    }

    pub fn delete(self) -> Result<()> {
        if Path::new(&self.path).exists() {
            fs::remove_dir_all(&self.path)
                .context("The database couldn't be erased because its folder does not exist")?;
        }
        Ok(())
    }

    pub fn upload_file(&self, file: &TagFile) -> Result<TagFile> {
        // Hardlinks the file we want to the tagmaid database path
        info!(
            "TagDatabase - upload_file() - Uploading/hardlinking file {} to filesystem",
            &file.display()
        );

        info!("TagDatabase - upload_file() - Getting unix timestamp for now");
        let now_unix_timestamp = Utc::now().timestamp();

        info!("TagDatabase - upload_file() - Getting a trimmed hash of the file");
        let trimmed_hash_hex = crate::data::tag_util::trimmed_hash_hex(&file.file_hash)?;
        let mut db_files_path = self.path.clone();
        db_files_path.push("files");
        db_files_path.push(format!(
            "{}-{}-{}",
            now_unix_timestamp,
            trimmed_hash_hex,
            &file.get_file_name_from_path()
        ));

        hardlink_file_else_copy(&file.get_path(), &db_files_path)?;

        let mut new_tagfile = TagFile::initialise_from_path(&db_files_path)?;

        // Keep the old file name
        new_tagfile.file_name = (&file.get_file_name_from_path()).to_string();

        Ok(new_tagfile)
    }

    pub fn remove_file(&self, file: &TagFile) -> Result<()> {
        info!("TagDatabase - remove_file() - file: {}", &file.display());
        fs::remove_file(&file.get_path()).with_context(|| {
            format!(
                "Database: Couldn't remove file '{}' from filesystem",
                &file.path.display()
            )
        })?;
        let db: &SqliteDatabase = &self.sqlite_database;
        db.remove_file(file).with_context(|| {
            format!(
                "Database: Couldn't remove file '{}' from Sqlite database",
                &file.path.display()
            )
        })?;

        Ok(())
    }

    pub fn get_tagfile_from_hash(&self, hash: &Vec<u8>) -> Result<TagFile> {
        debug!("TagDatabase - get_tagfile_from_hash() - hash: {:?}", &hash);
        Ok(self
            .sqlite_database
            .get_tagfile_from_hash(hash)
            .with_context(|| {
                format!("Database: Couldn't get TagFile from file hash {:?}", &hash)
            })?)
    }

    pub fn get_hashes_from_tag(&self, tag: &str) -> Result<HashSet<Vec<u8>>> {
        info!("TagDatabase - get_hashes_from_tag() - tag: {}", &tag);
        let db: &SqliteDatabase = &self.sqlite_database;
        let hashes = db
            .get_hashes_from_tag(&tag)
            .with_context(|| format!("Database: Couldn't get hashes from tag {}", &tag))?;
        Ok(hashes)
    }

    pub fn get_all_file_hashes(&self) -> Result<HashSet<Vec<u8>>> {
        info!("TagDatabase - get_all_file_hashes()");
        let db: &SqliteDatabase = &self.sqlite_database;
        let hashes = db
            .get_all_file_hashes()
            .context("Database: Couldn't get all file hashes")?;
        Ok(hashes)
    }

    #[cfg(test)]
    pub fn create_random_tagfile() -> TagFile {
        // random 16-char string
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        let tmp_dir = tempfile::tempdir().unwrap();
        // into_path is necessary for tempdir to persist in the file system
        let tmp_file_path = tmp_dir.into_path().as_path().join(random_string);

        let mut temp_file = File::create(&tmp_file_path).unwrap();
        let random_string_2 = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        let _ = &temp_file.write_all(random_string_2.as_bytes()).unwrap();

        let tagfile = TagFile::initialise_from_path(&tmp_file_path).unwrap();
        return tagfile;
    }

    // pub fn cleanup(&self) -> Result<()> {
    //     // Stage 1: Mark for cleanup
    //     let file_hashes = &self.get_all_file_hashes()?;
    //     let mut hashes_to_clean_up = HashSet::new();
    //     for hash in file_hashes {
    //         let tags = &self.get_tags_from_hash(&hash)?;
    //         if tags.is_empty() {
    //             // There are no tags on the file, mark hash for cleanup
    //             info!(
    //                 "TagDatabase - cleanup() - Marking file with hash {:?} to deletion",
    //                 &hash
    //             );
    //             hashes_to_clean_up.insert(hash);
    //         }
    //     }

    //     // Stage 2: Cleanup
    //     for hash_to_clean in hashes_to_clean_up {
    //         let file_to_clean: &TagFile = &self.get_tagfile_from_hash(hash_to_clean)?;
    //         info!(
    //             "TagDatabase - cleanup() - Removing useless file {}",
    //             &file_to_clean.display()
    //         );
    //         let _ = &self.remove_file(file_to_clean)?;
    //     }

    //     Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn create_random_tagdatabase() -> TagDatabase {
        let tmp_dir = tempfile::tempdir().unwrap();
        let tmp_path = tmp_dir.into_path();

        // random 16-char string
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        let db: TagDatabase = TagDatabase::initialise(random_string, Some(tmp_path)).unwrap();
        return db;
    }

    fn create_random_tagfile_in_tagdatabase() -> TagFile {
        let tmp_dir = tempfile::tempdir().unwrap();
        let tmp_path = tmp_dir.into_path();

        // random 16-char string
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        let db: TagDatabase = TagDatabase::initialise(random_string, Some(tmp_path)).unwrap();
        let tagfile = TagDatabase::create_random_tagfile();
        let uploaded_tagfile = db.upload_file(&tagfile).unwrap();
        return uploaded_tagfile;
    }

    #[test]
    fn should_tagfile_upload_in_fs() {
        let tagfile = create_random_tagfile_in_tagdatabase();
        let db = create_random_tagdatabase();

        let uploaded_tagfile = db.upload_file(&tagfile).unwrap();
        assert_eq!(tagfile.file_hash, uploaded_tagfile.file_hash);
        assert_eq!(tagfile.tags, uploaded_tagfile.tags);
        assert!(uploaded_tagfile.get_path().is_file() && uploaded_tagfile.get_path().exists());
    }

    #[test]
    fn should_tagfile_remove_in_fs() {
        let tagfile = TagDatabase::create_random_tagfile();
        let db = create_random_tagdatabase();
        let uploaded_tagfile = db.upload_file(&tagfile).unwrap();
        let tagfile_path = uploaded_tagfile.get_path();

        assert!(tagfile_path.is_file() && tagfile_path.exists());
        db.remove_file(&uploaded_tagfile).unwrap();
        assert!(!tagfile_path.is_file());
        assert!(!tagfile_path.exists());
        assert!(!tagfile_path.is_file() && !tagfile_path.exists());
    }

    #[test]
    fn should_create_database() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let tmp_path = tmp_dir.into_path();

        // random 16-char string
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        let mut db_path = tmp_path.clone();
        db_path.push("tag-maid");
        db_path.push(&random_string);

        let db: TagDatabase =
            TagDatabase::initialise(random_string.to_string(), Some(tmp_path)).unwrap();

        // asserts we created the database properly by checking
        // the folder is there
        assert!(Path::new(&db_path).exists());

        assert_eq!(&db.path, &db_path);
        assert_eq!(&db.name, &random_string);

        // only 2 files: sqlite.db and "files" folder
        assert_eq!(db.contents.count(), 2);

        fs::remove_dir_all(&db_path).unwrap();

        // switch to a different path
        db_path.pop();
        let test_database_string_2 = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        db_path.push(test_database_string_2);

        // should not exist since there is no database there
        assert!(!Path::new(&db_path).exists());
    }

    #[test]
    fn should_delete_database() {
        let db = create_random_tagdatabase();
        // database is created. Path tag-maid/<random-name> exists
        let db_path = &db.path.clone();
        assert!(Path::new(&db_path).exists());
        db.delete().ok();

        // now the path should have been completely deleted
        assert!(!Path::new(&db_path).exists());
    }

    // #[test]
    // fn should_tags_in_database_update() {
    //     let db = create_random_tagdatabase();
    //     let mut file = create_random_tagfile();

    //     let mut file_tags = HashSet::new();

    //     file_tags.insert("cool".to_string());
    //     file_tags.insert("amazing".to_string());
    //     file_tags.insert("epic_fail".to_string());

    //     let _ = &file.add_tags(&file_tags).unwrap();
    //     // add file
    //     db.update_file(&file).unwrap();
    //     let old_tags_from_db = db.get_tags_from_hash(&file.file_hash).unwrap();
    //     assert_eq!(&old_tags_from_db, &file.tags);
    //     // change some tags
    //     file.remove_tag("epic_fail").ok();
    //     file.add_tag("awesome_tag").ok();

    //     // update file already in database with the newer tags
    //     db.update_file(&file).unwrap();
    //     let new_tags_from_db = db.get_tags_from_hash(&file.file_hash).unwrap();

    //     // test: the updated tags should be given back
    //     assert_eq!(&new_tags_from_db, &file.tags);
    // }

    // #[test]
    // fn should_database_file_get_added_and_deleted() {
    //     let db = create_random_tagdatabase();

    //     let mut file1 = create_random_tagfile();
    //     let mut file1_tags = HashSet::new();

    //     file1_tags.insert("cool".to_string());
    //     file1_tags.insert("amazing".to_string());
    //     file1_tags.insert("epic_fail".to_string());

    //     let _ = &file1.add_tags(&file1_tags).unwrap();

    //     // add file 1 to db
    //     db.update_file(&file1).unwrap();

    //     // test: do tag tables in the database truly contain file1's hash?
    //     assert!(db
    //         .get_hashes_from_tag("cool")
    //         .unwrap()
    //         .contains(&file1.file_hash));
    //     assert!(db
    //         .get_hashes_from_tag("amazing")
    //         .unwrap()
    //         .contains(&file1.file_hash));
    //     assert!(db
    //         .get_hashes_from_tag("epic_fail")
    //         .unwrap()
    //         .contains(&file1.file_hash));

    //     let file_from_db = db.get_tagfile_from_hash(&file1.file_hash).unwrap();

    //     // test: the hash of the file we are adding should be the hash stored in the database
    //     assert_eq!(&file_from_db.file_hash, &file1.file_hash);

    //     // add another different file to db
    //     let mut file2 = create_random_tagfile();
    //     let _ = &file2.add_tag("cool");
    //     db.update_file(&file2).unwrap();

    //     // remove file1 from db, now only file2 is there
    //     db.remove_file(&file1).unwrap();

    //     // test: file1 shouldn't exist anymore and it will return an Err
    //     assert!(db.get_tagfile_from_hash(&file1.file_hash).is_err());

    //     // test: the file hash shouldn't be contained in the tag tables anymore
    //     println!("{:?}", db.get_hashes_from_tag("cool").unwrap());

    //     // the "cool" tag table still contains the hash from FILE 2 but not the one from FILE 1
    //     assert!(db
    //         .get_hashes_from_tag("cool")
    //         .unwrap()
    //         .contains(&file2.file_hash));
    //     assert!(!db
    //         .get_hashes_from_tag("cool")
    //         .unwrap()
    //         .contains(&file1.file_hash));

    //     // SQL will delete those tables since they only contained the one file hash we deleted
    //     // so the function will Err()
    //     assert!(!db.get_hashes_from_tag("amazing").is_err());
    //     assert!(!db.get_hashes_from_tag("epic_fail").is_err());
    // }
}
