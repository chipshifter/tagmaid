use std::path::PathBuf;

use crate::{data::ui_util::create_image_thumbnail, TAGMAID_DATABASE};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    pub image_path: String,
    pub file_name: String,
}

impl FileResult {
    pub fn from_file_hash(file_hash: &Vec<u8>) -> Result<Self> {
        let tf = &TAGMAID_DATABASE
            .get_tagfile_from_hash(file_hash)
            .context("Couldn't find TagFile with specificied file hash")?;

        Ok(FileResult {
            image_path: create_image_thumbnail(&tf.path, 100, 100)
                .into_os_string()
                .into_string()
                .ok()
                .context("Couldn't convert tag file's image thumbnail path to String")?,
            file_name: tf.file_name.clone(),
        })
    }
}
