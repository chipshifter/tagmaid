use crate::data::cache::TagMaidCache;
use crate::database::{sqlite_tags::TagsDatabase, tagmaid_database::TagMaidDatabase, sqlite_taginfo::TagInfoDatabase};
use crate::FsDatabase;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub struct TagInfo {
    pub tag: String,
    pub upload_count: i64,
}

impl TagInfo {
    pub fn new(tag: String) -> TagInfo {
        TagInfo {
            tag: tag,
            upload_count: 0
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
    fn should_get_tag_info_attributes() {
        let tag_info = TagInfo {
            tag: "test".to_string(),
            upload_count: 1337,
        };

        assert_eq!(tag_info.get_tag(), "test".to_string());
        assert_eq!(tag_info.get_upload_count(), 1337);
    }

    #[test]
    fn should_tag_info_initialise() {
        let tag_info = TagInfo {
            tag: "test".to_string(),
            upload_count: 0,
        };

        assert_eq!(TagInfo::new("test".to_string()), tag_info);
    }
}
