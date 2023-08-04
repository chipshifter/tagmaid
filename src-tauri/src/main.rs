#![allow(dead_code, unused_imports)]
pub mod client;
pub mod data;
pub mod database;
pub mod feature_flags;
pub mod tauri_error;
use crate::data::{config::Config, tag_file::TagFile};
use crate::database::{filesystem::FsDatabase, tagmaid_database::TagMaidDatabase};
use crate::feature_flags::FeatureFlags;
use anyhow::{anyhow, Result};
use client::results::FileResult;
use image::EncodableLayout;
use once_cell::sync::Lazy;
use tauri_error::{TauriError, TauriResult};
#[macro_use]
extern crate log;

// Search tab
#[tauri::command]
fn do_search(query: &str) -> TauriResult<Vec<String>, TauriError> {
    // Gotta convert the Vec<Vec<u8>> into Vec<String> for serde purposes
    let results: Vec<String> =
        client::search::do_search(query).map_err(|e| TauriError::Error(e))?
        .iter()
        .filter_map(|vec| serde_json::to_string(vec).ok())
        .collect();

    Ok(results)
}

// Results tab
#[tauri::command]
fn get_result(file_hash: &str) -> TauriResult<FileResult, TauriError> {
    let file_hash_bytes: Vec<u8> =
        serde_json::from_str(file_hash).map_err(|e| TauriError::Error(anyhow!(e)))?;
    client::results::FileResult::from_file_hash(&file_hash_bytes).map_err(|e| TauriError::Error(e))
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// I can already hear the dogs barking
static TAGMAID_DATABASE: Lazy<TagMaidDatabase> =
    Lazy::new(|| crate::database::tagmaid_database::init());

/** The main function.

It does the following in order:
 - Initialises the logger (`env_logger`).
 - If specified, imports the files located in `src/samples` with a hardcoded tag
 (this will be changed in the future)
 - If specified, runs a function meant to be used for handling more database things at
 startup
 - Loads [`Config`](crate::data::config)
 - Opens Tauri UI
*/
fn main() {
    env_logger::init();
    log::info!("Starting up TagMaid. Hello!");

    #[cfg(feature = "import_samples")]
    let _import = import_samples(Lazy::force(&TAGMAID_DATABASE)).unwrap();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![do_search, get_result])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    //let cfg = Config::load();
}

#[cfg(feature = "import_samples")]
fn import_samples(db: &TagMaidDatabase) -> Result<()> {
    let paths = std::fs::read_dir("src/sample").unwrap();

    for path in paths {
        let path_path = path.as_ref().unwrap().path().clone();
        if (&path.unwrap().metadata().unwrap().is_file()).to_owned() {
            println!("Adding file {} to db", &path_path.display());
            let mut file = TagFile::initialise_from_path(&path_path)?;
            file.add_tag("test")?;
            db.update_tagfile(&file)?;
        }
    }
    Ok(())
}

#[cfg(feature = "manual")]
fn manual_db(db: &TagMaidDatabase) -> Result<()> {
    // One file, many tags (in order too)
    let mut file = TagFile::initialise_from_path(Path::new("src/sample/toes.png"))?;
    for i in 0..150 {
        file.add_tag(format!("tag{i}").as_ref())?;
    }
    db.update_tagfile(&file)?;
    Ok(())
}
