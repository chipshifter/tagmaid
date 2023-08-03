//! SqliteDatabase is the main component for handling the database connection.
//! The database is stored as the `sqlite.db` file located in the main
//! [`TagMaidDatabase`](crate::database::tagmaid_database) path. If you want
//! more information on the database internals, visit
//! [`SqliteDatabase`](crate::database::sqlite_database::SqliteDatabase).
//!
//! The actual database is structure in the following way. There are as of yet 3 tables:
//!
//! - `_files`, for storing information about files (with [`TagFile`](crate::data::tag_file::TagFile)).
//! It is being handled by functions in [`sqlite_files`](crate::database::sqlite_files).
//!
//! - `_tags`, for storing information about tags (with [`TagInfo`](crate::data::tag_info::TagInfo)).
//! It is being handled by functions in [`sqlite_taginfo`](crate::database::sqlite_taginfo) and
//! [`sqlite_tags`](crate::database::sqlite_tags).
//!
//! - One table for each tag, named after them (with no underscores), which stores information about the file hashes
//! using the tag. It is being currently handled by functions in [`sqlite_files`](crate::database::sqlite_files).
//!
//! You should however probably deal with everything through the functions given in
//! [`TagMaidDatabase`](crate::database::tagmaid_database).
use crate::data::{tag_file::TagFile, tag_info::TagInfo};
use crate::database::{sqlite_files::FilesDatabase, sqlite_tags::TagsDatabase};
use anyhow::{bail, Context, Result};
use chrono::{DateTime, Utc};
use log::*;
#[cfg(test)]
use rand::distributions::{Alphanumeric, DistString};
use rusqlite::Connection;
use std::collections::HashSet;
use std::fs::{self, File, ReadDir};
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

/** The database object containing the connection.

It has 2 types of tables:

1) The `_files` table, which contains information about the individually
    uploaded files. A row would have the following information: `file_name`,
    `file_path`, `file_hash`, `upload_date`, and `tags`.
2) One table for each tag. Each row in these tables contain the hashes of
    the files which are linked to this tag. These tables are used for searching.
*/
pub struct SqliteDatabase {
    db: Connection,
}

impl SqliteDatabase {
    /// Opens the connection to the database at a given path. The `name` path is the name
    /// of the parent folder which will contain `sqlite.db` (and the uploaded files).
    pub fn initialise(db_path: &PathBuf) -> Result<SqliteDatabase> {
        info!("SqliteDatabase - initialise_default() - Initialising default database");
        let mut path = db_path.clone();
        path.push("sqlite.db");
        debug!(
            "SqliteDatabase - initialise_default() - Opening connection to database at path {}",
            &path.display()
        );

        Ok(SqliteDatabase {
            db: Self::open_db_connection(&path)?,
        })
    }

    pub fn from(db: Connection) -> SqliteDatabase {
        SqliteDatabase { db }
    }

    pub fn open_db_connection(sqlite_file_path: &PathBuf) -> Result<Connection> {
        let db = Connection::open(sqlite_file_path)
            .context("Couldn't open a connection to SQLite database")?;

        FilesDatabase::create_files_table(&db)?;
        TagsDatabase::create_tags_table(&db)?;

        Ok(db)
    }

    pub fn get_connection(&self) -> &Connection {
        &self.db
    }

    #[cfg(test)]
    pub fn get_random_db_connection() -> SqliteDatabase {
        let mut tmp_db_dir = tempfile::tempdir().unwrap().into_path();
        let random_db_name = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        tmp_db_dir.push(random_db_name);
        let _ = File::create(&tmp_db_dir).unwrap();
        let tmp_db = SqliteDatabase::open_db_connection(&tmp_db_dir).unwrap();
        return SqliteDatabase { db: tmp_db };
    }
}
