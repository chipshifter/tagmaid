#![allow(dead_code, unused_imports)]
pub mod data;
pub mod database;
pub mod feature_flags;
use crate::data::{config::Config, tag_file::TagFile};
use crate::database::{filesystem::FsDatabase, tagmaid_database::TagMaidDatabase};
use crate::feature_flags::FeatureFlags;
use anyhow::{bail, Context, Result};
use image::EncodableLayout;
#[macro_use]
extern crate log;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/** The main function.

It does the following in order:
 - Initialises logging (`env_logger`).
 - If specified, imports the files located in `src/samples` with a hardcoded tag
 (this will be changed in the future)
 - If specified, runs a function meant to be used for handling more database things at
 startup
 - Loads [`Config`](crate::data::config)
 - Launches [`app_main`](crate::app_main) which creates and opens the egui UI.
*/
fn main() {
    env_logger::init();
    log::info!("Starting up TagMaid. Hello!");

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    //let cfg = Config::load();
}

#[derive(Clone)]
pub struct UIData {
    pub db: TagMaidDatabase,
    pub search_results_hashes: Vec<Vec<u8>>,
}

impl UIData {
    pub fn new(db: TagMaidDatabase) -> Self {
        Self {
            db: db,
            search_results_hashes: Vec::new(),
        }
    }

    pub fn db(&self) -> TagMaidDatabase {
        self.db.clone()
    }

    pub fn update_search_results(&mut self, new_vector: Vec<Vec<u8>>) {
        self.search_results_hashes = new_vector.clone();
    }

    pub fn get_search_results(&self) -> Vec<Vec<u8>> {
        self.search_results_hashes.clone()
    }
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
