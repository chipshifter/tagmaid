use crate::data::cache::TagMaidCache;
use crate::database::{tagmaid_database::TagMaidDatabase, sqlite_tags::TagsDatabase};
use crate::FsDatabase;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub struct TagInfo {
    pub tag: String,
    pub upload_count: i64,
}

impl TagInfo {
    pub fn initialise(tag: String, db: &TagMaidDatabase) -> TagInfo {
        // TODO: Move tag db stuff to another separate thing
        let sql_db_mutex = db.get_sql_db();
        let sql_db = sql_db_mutex.lock().unwrap();

        match db.get_cache().get_tag_info(&tag) {
            Some(tag_info) => tag_info,
            None => {
                let tag_info =
                    TagsDatabase::get_tag_info(sql_db.get_connection(), &tag).unwrap_or(TagInfo {
                        tag: tag,
                        upload_count: 0,
                    });

                db.get_cache().cache_tag_info(tag_info.clone()).ok();

                return tag_info;
            }
        }
    }

    pub fn get_tag(&self) -> String {
        self.tag.to_owned()
    }

    pub fn get_upload_count(&self) -> i64 {
        self.upload_count.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_taginfo_attributes() {
        let tag_info = TagInfo {
            tag: "test".to_string(),
            upload_count: 1337,
        };

        assert_eq!(tag_info.get_tag(), "test".to_string());
        assert_eq!(tag_info.get_upload_count(), 1337);
    }

    #[test]
    fn should_taginfo_initialise() {
        let db = TagMaidDatabase::create_random_tagmaiddatabase();

        let tag_info = TagInfo {
            tag: "test".to_string(),
            upload_count: 0,
        };

        assert_eq!(TagInfo::initialise("test".to_string(), &db), tag_info);
    }
}
