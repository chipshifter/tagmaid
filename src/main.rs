#![allow(dead_code, unused_imports)]
pub mod data;
pub mod database;
pub mod feature_flags;
pub mod ui;
use std::sync::{Arc, Mutex};

use crate::data::{config::Config, tag_file::TagFile};
use crate::database::{filesystem::FsDatabase, tagmaid_database::TagMaidDatabase};
use crate::feature_flags::FeatureFlags;
use anyhow::{bail, Context, Result};
use image::EncodableLayout;
#[macro_use]
extern crate log;

use dioxus::prelude::*;

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
fn main() -> Result<()> {
    env_logger::init();
    info!("Starting up TagMaid. Hello!");

    if FeatureFlags::DIOXUS_UI {
        dioxus::launch(app);
    }

    //let cfg = Config::load();
    Ok(())
}

#[derive(Clone)]
pub struct UIData {
    pub db: TagMaidDatabase,
    pub search_results_hashes: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl UIData {
    pub fn new(db: TagMaidDatabase) -> Self {
        Self {
            db: db,
            search_results_hashes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn db(&self) -> TagMaidDatabase {
        self.db.clone()
    }

    pub fn update_search_results(&mut self, new_vector: Vec<Vec<u8>>) {
        *self.search_results_hashes.lock().unwrap() = new_vector.clone();
    }

    pub fn get_search_results(&self) -> Vec<Vec<u8>> {
        self.search_results_hashes.lock().unwrap().clone()
    }
}

#[derive(Clone)]
pub struct UITagmaidDatabase(TagMaidDatabase);

fn get_ui_data() -> crate::UIData {
    use_root_context::<crate::UIData>(|| panic!() )
}

/// dioxus
fn app() -> Element {
    // TODO : change the db thing
    let db: TagMaidDatabase = crate::database::tagmaid_database::init();
    #[cfg(feature = "import_samples")]
    import_samples(&db);

    // Shared shate of TagMaidDatabase
    let db = use_root_context(|| UITagmaidDatabase(db.clone()));

    // TODO: Independent shared states for each little thing
    let _ui = use_root_context(|| UIData::new(db.0.clone()));
    rsx! {
        style { {include_str!("ui/css/root.css")} }
        crate::ui::render {}
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
