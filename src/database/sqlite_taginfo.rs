//! The database interface for handling [`TagInfo`](crate::data::tag_info) objects
//! in the SQLite database (stored in the `_tags` table). It is higher level than
//! `sqlite_tags` which doesn't handle TagInfo but raw data (and is deprecated for use).

use crate::data::{tag_file::TagFile, tag_info::TagInfo};
use crate::database::{sqlite_database::SqliteDatabase, sqlite_files::FilesDatabase};
use anyhow::{bail, Context, Result};
use log::*;
use rusqlite::Connection;
use std::collections::{BTreeMap, HashSet};

pub struct TagInfoDatabase;

impl TagInfoDatabase {
    /// Adds tag info. If the tag info is new then the upload count should probably be equal to 0
    pub fn add_tag_info(db: &Connection, tag_info: &TagInfo) -> Result<()> {
        db.execute(
            "INSERT INTO _tags (tag_name, upload_count) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
            (tag_info.get_tag(), tag_info.get_upload_count()),
        )
        .context("Couldn't add tag {tag} in _tags table")?;
        Ok(())
    }

    pub fn get_tag_info_from_tag(db: &Connection, tag: &str) -> Result<TagInfo> {
        return Ok(db.query_row(
            "SELECT upload_count FROM _tags WHERE tag_name IS :tag",
            &[(":tag", tag)],
            |row| {
                Ok(TagInfo {
                    // Right now, just a copy of set_tag_count() made to work with TagInfo
                    tag: tag.to_owned(),
                    upload_count: row.get(0)?,
                })
            },
        )?);
    }

    pub fn update_tag_info(db: &Connection, tag_info: &TagInfo) -> Result<()> {
        // Update upload count
        db.execute(
            "UPDATE _tags SET upload_count = ?1 WHERE tag_name IS ?2",
            rusqlite::params![tag_info.get_upload_count(), tag_info.get_tag()],
        )
        .context("Couldn't update tag count for tag {tag} in database")?;
        Ok(())
    }

    pub fn remove_tag_info(db: &Connection, tag: &str) -> Result<()> {
        db.execute("DELETE FROM _tags WHERE tag_name IS :tag", &[(":tag", tag)])
            .context("Couldn't remove tag {tag} in _tags table")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::sqlite_tags::TagsDatabase;
    use crate::TagMaidDatabase;

    #[test]
    fn should_upload_and_update_tag_info() {
        let sql_db = SqliteDatabase::get_random_db_connection();
        let db = sql_db.get_connection();

        let mut tag_info = TagInfo {
            tag: "test".to_string(),
            upload_count: 1337,
        };

        assert!(TagsDatabase::create_tags_table(&db).is_ok());
        assert!(TagInfoDatabase::add_tag_info(&db, &tag_info).is_ok());
        assert_eq!(
            TagInfoDatabase::get_tag_info_from_tag(&db, "test").unwrap(),
            tag_info
        );

        tag_info.upload_count = 1338;

        assert!(TagInfoDatabase::update_tag_info(&db, &tag_info).is_ok());
        assert_eq!(
            TagInfoDatabase::get_tag_info_from_tag(&db, "test").unwrap(),
            tag_info
        );
    }
}
