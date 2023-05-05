//! `FsDatabase` is the interface for managing file-system related operations on the database.
use crate::data::tag_file::TagFile;
use crate::database::sqlite_database::SqliteDatabase;
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

#[derive(Debug)]
pub struct FsDatabase {
    pub name: String,
    pub path: PathBuf,
    pub contents: ReadDir,
}

/** Hardlinks a file from a given path (`old_path`) to a new path (`new_path`). If hardlinking
fails (this can happen if the two files are on different filesystems or root path), then it
it attempt to do a copy instead.

Hardlinking means that instead of duplicating files each time, which would use storage for nothing,
it creates a reference to the same file inode, which gives you two files (the old one and new one) that
point to the *same* data on the disk. Therefore very little extra storage is used (except for storing metadata).  
*/
fn hardlink_file_else_copy(old_path: &PathBuf, new_path: &PathBuf) -> Result<()> {
    info!(
        "hardlink_file_else_copy() - Hardlinking file from old path {} to new path {}",
        &old_path.display(),
        &new_path.display()
    );
    if fs::hard_link(old_path, new_path).is_err() {
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
    pub fn initialise(path: &PathBuf) -> Result<FsDatabase> {
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
        let db_folder = fs::read_dir(&path)?;

        Ok(FsDatabase {
            // A permanent solution to a temporary problem
            name: path
                .file_name()
                .expect("blah blah utf-8")
                .to_str()
                .unwrap_or("frank")
                .to_owned(),
            path: path.clone(),
            contents: db_folder,
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

    #[cfg(test)]
    pub fn create_random_fsdatabase() -> FsDatabase {
        let tmp_dir = tempfile::tempdir().unwrap();
        let mut tmp_path = tmp_dir.into_path();

        // random 16-char string
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        tmp_path.push(random_string);

        let db: FsDatabase = FsDatabase::initialise(&tmp_path).unwrap();
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
    fn should_create_database() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let tmp_path = tmp_dir.into_path();

        // random 16-char string
        let random_string = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

        let mut db_path = tmp_path.clone();
        db_path.push(&random_string);

        let db: FsDatabase = FsDatabase::initialise(&db_path).unwrap();

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
