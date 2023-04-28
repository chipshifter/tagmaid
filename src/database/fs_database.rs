//! FsDatabase is the old database interface. It is in the process of being repurposed as the
//! "filesystem" interface, used for hardlinking files to the database path etc.
use crate::data::tag_file::TagFile;
use crate::database::sqlite_database::{SqliteDatabase, TagFileSqlite};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use log::*;
#[cfg(test)]
use rand::distributions::{Alphanumeric, DistString};
use rusqlite::Connection;
use std::collections::HashSet;
use std::fs::{self, File, ReadDir};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct FsDatabase {
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

impl FsDatabase {
    pub fn initialise(name: String, custom_path: Option<PathBuf>) -> Result<FsDatabase> {
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
        Ok(FsDatabase {
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
            "FsDatabase - upload_file() - Uploading/hardlinking file {} to filesystem",
            &file.display()
        );

        info!("FsDatabase - upload_file() - Getting unix timestamp for now");
        let now_unix_timestamp = Utc::now().timestamp();

        info!("FsDatabase - upload_file() - Getting a trimmed hash of the file");
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
        info!("FsDatabase - remove_file() - file: {}", &file.display());
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
        debug!("FsDatabase - get_tagfile_from_hash() - hash: {:?}", &hash);
        Ok(self
            .sqlite_database
            .get_tagfile_from_hash(hash)
            .with_context(|| {
                format!("Database: Couldn't get TagFile from file hash {:?}", &hash)
            })?)
    }

    pub fn get_hashes_from_tag(&self, tag: &str) -> Result<HashSet<Vec<u8>>> {
        info!("FsDatabase - get_hashes_from_tag() - tag: {}", &tag);
        let db: &SqliteDatabase = &self.sqlite_database;
        let hashes = db
            .get_hashes_from_tag(&tag)
            .with_context(|| format!("Database: Couldn't get hashes from tag {}", &tag))?;
        Ok(hashes)
    }

    pub fn get_all_file_hashes(&self) -> Result<HashSet<Vec<u8>>> {
        info!("FsDatabase - get_all_file_hashes()");
        let db: &SqliteDatabase = &self.sqlite_database;
        let hashes = db
            .get_all_file_hashes()
            .context("Database: Couldn't get all file hashes")?;
        Ok(hashes)
    }

    #[cfg(test)]
    pub fn create_random_fsdatabase() -> FsDatabase {
        let tmp_dir = tempfile::tempdir().unwrap();
        let tmp_path = tmp_dir.into_path();

        // random 16-char string
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        let db: FsDatabase = FsDatabase::initialise(random_string, Some(tmp_path)).unwrap();
        return db;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::{Alphanumeric, DistString};

    #[test]
    fn should_tagfile_upload_in_fs() {
        let tagfile = TagFile::create_random_tagfile_in_fsdatabase();
        let db = FsDatabase::create_random_fsdatabase();

        let uploaded_tagfile = db.upload_file(&tagfile).unwrap();
        assert_eq!(tagfile.file_hash, uploaded_tagfile.file_hash);
        assert_eq!(tagfile.tags, uploaded_tagfile.tags);
        assert!(uploaded_tagfile.get_path().is_file() && uploaded_tagfile.get_path().exists());
    }

    #[test]
    fn should_tagfile_remove_in_fs() {
        let tagfile = TagFile::create_random_tagfile();
        let db = FsDatabase::create_random_fsdatabase();
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

        let db: FsDatabase =
            FsDatabase::initialise(random_string.to_string(), Some(tmp_path)).unwrap();

        // asserts we created the database properly by checking
        // the folder is there
        assert!(Path::new(&db_path).exists());

        assert_eq!(&db.path, &db_path);
        assert_eq!(&db.name, &random_string);

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
        let db = FsDatabase::create_random_fsdatabase();
        // database is created. Path tag-maid/<random-name> exists
        let db_path = &db.path.clone();
        assert!(Path::new(&db_path).exists());
        db.delete().ok();

        // now the path should have been completely deleted
        assert!(!Path::new(&db_path).exists());
    }

    #[test]
    fn should_db_structure_be_correct() {
        let db = FsDatabase::create_random_fsdatabase();

        // The database when initialised creates the following things in the parent folder:
        //  - sqlite.db for the SQLite database
        //  - A "files" folder that contains all the uploaded files

        // db.contents is a ReadDir reading from the database path.
        // We turn that into a vector of PathBufs to compare with what is expected
        let path_iter: Vec<PathBuf> = db.contents.map(|f| f.unwrap().path()).collect();

        // "files"
        let mut files_path = db.path.clone();
        files_path.push("files");

        // "sqlite.db"
        let mut sqlite_db_file_path = db.path.clone();
        sqlite_db_file_path.push("sqlite.db");

        // Check if files in db.contents
        assert_eq!(path_iter, vec![files_path, sqlite_db_file_path]);
    }

    /// We upload a bunch of random files and see if they're here
    #[test]
    fn should_files_upload_in_db() {
        use rand::Rng;

        let db = FsDatabase::create_random_fsdatabase();

        let n_files = rand::thread_rng().gen_range(2..20);
        for _i in 0..n_files {
            let random_tagfile = crate::TagFile::create_random_tagfile();
            let _ = db.upload_file(&random_tagfile);
        }

        let mut files_path = db.path;
        files_path.push("files");

        assert_eq!(files_path.read_dir().unwrap().count(), n_files);
    }
}
