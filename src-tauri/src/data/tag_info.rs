//! TagInfo is an object which stores information/properties about tags.
//! Currently the only proprety stored is the upload count, i.e. the amount
//! of files in the database which contain the tag.
use crate::data::cache::TagMaidCache;
use crate::database::{
    sqlite_taginfo::TagInfoDatabase, sqlite_tags::TagsDatabase, tagmaid_database::TagMaidDatabase,
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub struct TagInfo {
    pub tag: String,
    pub upload_count: i64,
}

impl TagInfo {
    /// Takes a given `tag` string and returns an empty TagInfo object.
    /// The `upload_count` attribute is initialised to 0.
    pub fn new(tag: String) -> TagInfo {
        TagInfo {
            tag: tag,
            upload_count: 0,
        }
    }

    /// Returns an owned value of the `tag` attribute
    pub fn get_tag(&self) -> String {
        self.tag.to_owned()
    }

    /// Returns an owned value of the `upload_count` attribute
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
