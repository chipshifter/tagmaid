use crate::data::{tag_file::TagFile, tag_info::TagInfo};
use crate::database::{sqlite_database::SqliteDatabase};
use anyhow::{bail, Context, Result};
use log::*;
use rusqlite::Connection;

pub struct TagsDatabase;

impl TagsDatabase {
    /// Adds a tag in the `_tags` table. Used to retain some information about the tags
    /// themselves (for now, only the amount of files)
    /// If a tag is already present, nothing will change (and it will return Ok())
    pub fn add_tag(db: &Connection, tag: &str) -> Result<()> {
        db.execute(
            "INSERT INTO _tags (tag_name, upload_count) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
            (tag, 0),
        )
        .context("Couldn't add tag {tag} in _tags table")?;

        Ok(())
    }

    /// Removes a given `tag` from `_tags` table
    pub fn remove_tag(db: &Connection, tag: &str) -> Result<()> {
        db.execute("DELETE FROM _tags WHERE tag_name IS :tag", &[(":tag", tag)])
            .context("Couldn't remove tag {tag} in _tags table")?;

        Ok(())
    }

    /// TagInfo, we will use this for futureproof reasons
    pub fn get_tag_info(db: &Connection, tag: &str) -> Result<TagInfo> {
        return Ok(db.query_row(
            "SELECT upload_count FROM _tags WHERE tag_name IS :tag",
            &[(":tag", tag)],
            |row| {
                Ok(TagInfo {
                    tag: tag.to_owned(),
                    upload_count: row.get(0).unwrap_or(0),
                })
            },
        )?);
    }

    /// Retrieves the stored `upload_count` in `_tags` table. Note that it *could be* desynced
    /// with the actual amount of files that have `tag`. `upload_tag_count()` helps against this
    /// issue.
    pub fn get_tag_count(db: &Connection, tag: &str) -> Result<i64> {
        return Ok(db.query_row(
            "SELECT upload_count FROM _tags WHERE tag_name IS :tag",
            &[(":tag", tag)],
            |row| row.get(0),
        )?);
    }

    /// Sets `upload_count` attribute to `tag_count` on a given `tag` in the `_tags` table.
    /// This shouldn't be used manually, because it could desynchronise the count with the
    /// rest of the database. You probably want to use `increase_tag_count_by_one()`.
    pub fn set_tag_count(db: &Connection, tag: &str, tag_count: i64) -> Result<()> {
        db.execute(
            "UPDATE _tags SET upload_count = ?1 WHERE tag_name IS ?2",
            rusqlite::params![tag_count, tag],
        )
        .context("Couldn't update tag count for tag {tag} in database")?;

        Ok(())
    }

    /// Increases `upload_count` by one on a given `tag` in the `_tags` table. Used
    /// when uploading a new file.
    pub fn increase_tag_count_by_one(db: &Connection, tag: &str) -> Result<()> {
        let mut tag_count: i64 = Self::get_tag_count(db, tag)?;
        tag_count += 1;
        Self::set_tag_count(db, tag, tag_count)?;
        Ok(())
    }

    /// Synchronises the tag upload count with how many rows there
    /// are in a tag table (= number of files with the tag in db)
    /// It is a rather expensive call (it iterates though every row
    /// in the table you call to count) so it is not called often.
    pub fn sync_tag_count(db: &Connection, tag: &str) -> Result<()> {
        // Get the number of rows on a given tag table.
        // If it fails we assume there are no results.

        // Rusqlite's named parameters don't work for table names,
        // that's why I'm using format!().......
        let query = format!("SELECT COUNT(*) as count FROM {tag}");
        let count: i64 = db
            .query_row(query.as_str(), (), |row| row.get(0))
            .unwrap_or(0);

        // Update the tag upload count with what we just calculated
        let _ = Self::set_tag_count(db, tag, count)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn should_tag_entry_get_added() {
        let sql_db = SqliteDatabase::get_random_db_connection();
        let db = sql_db.get_connection();
        
        // Err() because the tag isn't there yet
        assert!(TagsDatabase::get_tag_count(db, "test").is_err());

        // Ok() because the 'set' updates if it finds anything. If it doesn't
        // it just moves on without errors
        assert!(TagsDatabase::set_tag_count(db, "test", 0).is_ok());

        // Err() because it relies on get_tag_count()
        assert!(TagsDatabase::increase_tag_count_by_one(db, "test").is_err());

        // We finally add the tag
        assert!(TagsDatabase::add_tag(db, "test").is_ok());
        assert_eq!(TagsDatabase::get_tag_count(db, "test").ok(), Some(0));
    }

    #[test]
    fn should_tag_entry_get_removed() {
        let sql_db = SqliteDatabase::get_random_db_connection();
        let db = sql_db.get_connection();

        assert!(TagsDatabase::add_tag(db, "test").is_ok());
        assert_eq!(TagsDatabase::get_tag_count(db, "test").ok(), Some(0));

        assert!(TagsDatabase::remove_tag(db, "test").is_ok());
        assert!(TagsDatabase::get_tag_count(db, "test").is_err());
    }

    #[test]
    fn should_tag_count_increase() {
        let sql_db = SqliteDatabase::get_random_db_connection();
        let db = sql_db.get_connection();
        
        assert!(TagsDatabase::add_tag(db, "test").is_ok());
        assert_eq!(TagsDatabase::get_tag_count(db, "test").ok(), Some(0));

        assert!(TagsDatabase::increase_tag_count_by_one(db, "test").is_ok());
        assert_eq!(TagsDatabase::get_tag_count(db, "test").ok(), Some(1));
    }

    #[test]
    fn should_tag_count_get_set() {
        let sql_db = SqliteDatabase::get_random_db_connection();
        let db = sql_db.get_connection();

        assert!(TagsDatabase::add_tag(db, "test").is_ok());
        assert_eq!(TagsDatabase::get_tag_count(db, "test").ok(), Some(0));

        assert!(TagsDatabase::set_tag_count(db, "test", 69).is_ok());
        assert_eq!(TagsDatabase::get_tag_count(db, "test").ok(), Some(69));
    }

    #[test]
    fn should_tag_entry_not_overwrite() {
        let sql_db = SqliteDatabase::get_random_db_connection();
        let db = sql_db.get_connection();

        assert!(TagsDatabase::add_tag(db, "test").is_ok());
        assert_eq!(TagsDatabase::get_tag_count(db, "test").ok(), Some(0));

        assert!(TagsDatabase::set_tag_count(db, "test", 69).is_ok());
        assert_eq!(TagsDatabase::get_tag_count(db, "test").ok(), Some(69));

        // Even though we re-add a tag that was already added, the count should
        // not reset to 0
        assert!(TagsDatabase::add_tag(db, "test").is_ok());
        assert_eq!(TagsDatabase::get_tag_count(db, "test").ok(), Some(69));
    }

    #[test]
    fn should_tag_count_sync() {
        use rand::Rng;

        let sql_db = SqliteDatabase::get_random_db_connection();
        let db = sql_db.get_connection();

        assert!(TagsDatabase::add_tag(db, "test").is_ok());

        // We add an arbritrary number
        assert!(TagsDatabase::set_tag_count(db, "test", 69).is_ok());
        assert_eq!(TagsDatabase::get_tag_count(db, "test").ok(), Some(69));

        // Since the db is new, it should sync back to 0
        assert!(TagsDatabase::sync_tag_count(db, "test").is_ok());
        assert_eq!(TagsDatabase::get_tag_count(db, "test").ok(), Some(0));

        // Add `n` random files
        let n_files = rand::thread_rng().gen_range(2..20);
        for _i in 0..n_files {
            // We add a random file with tag test
            let mut tmp_tagfile = crate::TagFile::create_random_tagfile();
            let _ = tmp_tagfile.add_tag("test");
            assert!(sql_db.add_file(&tmp_tagfile).is_ok());
            assert!(sql_db.update_tags_to_file(&tmp_tagfile).is_ok());
        }

        // Now there is 1 file with tag 'test' so syncing it back will `n`
        // (for the `n` files we added)
        assert!(TagsDatabase::sync_tag_count(db, "test").is_ok());
        assert_eq!(TagsDatabase::get_tag_count(db, "test").ok(), Some(n_files));
    }
}
