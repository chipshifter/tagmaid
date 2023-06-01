#![allow(dead_code, unused_imports)]
pub mod data;
pub mod database;
pub mod feature_flags;
pub mod ui;
use crate::data::{config::Config, tag_file::TagFile};
use crate::database::{filesystem::FsDatabase, tagmaid_database::TagMaidDatabase};
use crate::feature_flags::FeatureFlags;
use anyhow::{bail, Context, Result};
use image::EncodableLayout;
#[macro_use]
extern crate log;

use dioxus::prelude::*;
use once_cell::sync::Lazy;

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

    static DB: Lazy<TagMaidDatabase> = Lazy::new(|| crate::database::tagmaid_database::init());
    #[cfg(feature = "import_samples")]
    import_samples(&DB)?;

    // Used for manual database configuration
    // change the code as you see fit (for dev purposes)
    #[cfg(feature = "manual")]
    manual_db(&DB)?;

    if FeatureFlags::DIOXUS_UI {
        dioxus_desktop::launch_with_props(
            app,
            crate::ui::ui_new::UIData::new(&DB),
            dioxus_desktop::Config::default(),
        );
    }

    let cfg = Config::load();
    app_main(DB.clone(), cfg)?;
    Ok(())
}

/// dioxus
fn app(cx: Scope<crate::ui::ui_new::UIData>) -> Element {
    let ui_data = use_state(cx, || *cx.props);
    cx.render(rsx! {
        style { include_str!("ui/style.css") }
        crate::ui::ui_new::render(cx, ui_data) {}
    })
}

/// egui initialisation function
fn app_main(db: TagMaidDatabase, config: data::config::Config) -> Result<()> {
    let mut frame_options = eframe::NativeOptions::default();
    frame_options.drag_and_drop_support = true;
    frame_options.resizable = false;
    frame_options.initial_window_size = Some(egui::Vec2::new(800.0, 500.0));
    frame_options.default_theme = config.theme.egui_theme().unwrap();

    // Code for window png logo
    let dcode = image::open("logo.png")?;
    let size = (dcode.width(), dcode.height());
    let out = dcode.as_rgba8().unwrap().as_bytes();
    frame_options.icon_data = Some(eframe::IconData {
        width: size.0,
        height: size.1,
        rgba: out.to_owned(),
    });

    eframe::run_native(
        "Tag Maid",
        frame_options,
        Box::new(|cc| Box::new(ui::TagMaid::new(cc, db, config))),
    )
    .unwrap();
    Ok(())
}

#[cfg(feature = "import_samples")]
fn import_samples(db: &TagMaidDatabase) -> Result<()> {
    let paths = std::fs::read_dir("src/sample").unwrap();

    for path in paths {
        let path_path = path.as_ref().unwrap().path().clone();
        if (&path.unwrap().metadata().unwrap().is_file()).to_owned() {
            println!("Adding file {} to db", &path_path.display());
            let mut file = TagFile::initialise_from_path(&path_path)?;
            // Hardcoded don't care + ratio + stream Frank Ocean
            file.add_tag("frank_ocean")?;
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
