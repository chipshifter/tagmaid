use anyhow::{bail, Context, Result};
use egui::{
    epaint::text::TextWrapping,
    text::{LayoutJob, TextFormat},
    FontFamily, FontId, Vec2,
};
use image::EncodableLayout;

use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard, RwLock},
};

use crate::data::{
    self,
    config::{Config, Theme},
    search_command::Search,
    tag_file::TagFile,
};

use crate::database::{
    sqlite_database::SqliteDatabase, tag_database::TagDatabase, tagmaid_database::TagMaidDatabase,
};

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub enum TextureLabel {
    FileThumbnail(Arc<PathBuf>),
    UiImages(String),
}
impl TextureLabel {
    fn name(&self) -> &str {
        match self {
            Self::FileThumbnail(s) => s.to_str().unwrap_or("[Nonutf8 Texture Label]"),
            Self::UiImages(s) => &s,
        }
    }
    fn path(&self) -> Arc<PathBuf> {
        match self {
            Self::FileThumbnail(p) => p.clone(),
            Self::UiImages(s) => {
                let mut path = PathBuf::new();
                path.push("ui");
                path.push(s);
                path.set_extension("png");
                Arc::new(path)
            }
        }
    }
}

enum ViewPage {
    Add,
    Search,
    Results,
    #[cfg(feature = "ui_debug")]
    Debug,
    View,
    Edit,
    RemoveFile,
}
impl ViewPage {
    fn add(&self) -> bool {
        if let Self::Add = self {
            true
        } else {
            false
        }
    }
    fn search(&self) -> bool {
        if let Self::Search = self {
            true
        } else {
            false
        }
    }
    fn results(&self) -> bool {
        if let Self::Results = self {
            true
        } else {
            false
        }
    }
    #[cfg(feature = "ui_debug")]
    fn debug(&self) -> bool {
        if let Self::Debug = self {
            true
        } else {
            false
        }
    }
}

pub struct TagMaid {
    mode: ViewPage,
    thumbnail_paths: Arc<RwLock<HashMap<Vec<u8>, Arc<PathBuf>>>>,
    db: TagMaidDatabase,
    conf: Config,
    // Search
    search: String,
    search_err: Option<String>,
    search_options: Option<Search>,
    // Results
    update_search: Arc<Mutex<bool>>,
    results: Arc<Mutex<Vec<Vec<u8>>>>,
    // Add form
    add_path: Option<PathBuf>,
    path_future: Option<std::thread::JoinHandle<Option<PathBuf>>>,
    // View
    viewmode_tagfile_hash: Option<Vec<u8>>,
    // Edit
    edit_hash: Option<Vec<u8>>,
    edit_tags: BTreeSet<String>,
    edit_add_tags: String,
    // Remove
    remove_tagfile: Option<TagFile>,
}
impl TagMaid {
    pub fn new(_cc: &eframe::CreationContext<'_>, db: TagMaidDatabase, conf: Config) -> Self {
        Self {
            mode: ViewPage::Search,
            db: db,
            search: String::new(),
            results: Arc::new(Mutex::new(Vec::new())),
            update_search: Arc::new(Mutex::new(false)),
            search_err: None,
            search_options: None,
            thumbnail_paths: Arc::new(RwLock::new(HashMap::new())),
            add_path: None,
            path_future: None,
            conf: conf,
            viewmode_tagfile_hash: None,
            edit_hash: None,
            edit_tags: BTreeSet::new(),
            edit_add_tags: String::new(),
            remove_tagfile: None,
        }
    }

    fn load_image_rgba8(path: PathBuf) -> Result<image::RgbaImage> {
        let load_img = image::open(path);
        match load_img {
            Ok(img) => {
                return Ok(img.into_rgba8());
            }
            Err(_) => {
                // Probably unknown file format: Load placeholder
                let img = image::open(PathBuf::from("ui/file.png"))
                    .context("UI: Couldn't load placeholder result image")?;
                return Ok(img.into_rgba8());
            }
        }
    }

    /// Open an image into a texture that can be handled and shown in `egui`.
    fn get_texture(&self, ctx: &egui::Context, label: &TextureLabel) -> egui::TextureHandle {
        // Tries to find cache
        match self.db.get_cache().get_thumbnail(label) {
            Some(texture) => {
                debug!("Got cached search for texture with label {:?}", label);
                return texture.to_owned();
            }
            None => {}
        }

        // No cache, moving on

        let path = label.path();
        // If does path not exist or whatever it switches to placeholder
        let img = Self::load_image_rgba8(path.to_path_buf()).unwrap();
        let size = (img.width(), img.height());
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [size.0 as usize, size.1 as usize],
            img.as_bytes(),
        );
        let handle = ctx.load_texture(label.name(), image, egui::TextureOptions::NEAREST);

        // Caches thumbnail for next time
        match self
            .db
            .get_cache()
            .cache_thumbnail(label.clone(), handle.clone())
        {
            Ok(()) => {}
            Err(err) => {
                info!("WARNING: get_texture(): Couldn't open cache as mutable because it was already being borrowed: {err}");
            }
        }

        return handle;
    }

    fn get_thumbnail_path(
        thumbnail_paths: Arc<RwLock<HashMap<Vec<u8>, Arc<PathBuf>>>>,
        tagfile: Arc<TagFile>,
    ) -> Arc<PathBuf> {
        let hash = &tagfile.file_hash;
        if thumbnail_paths.read().unwrap().get(hash).is_none() {
            let path = tagfile.get_thumbnail_path();
            thumbnail_paths
                .write()
                .unwrap()
                .insert(hash.clone(), path.into());
        }
        thumbnail_paths.read().unwrap()[hash].clone()
    }

    /// Obtain results from a given search query `se`. Designed to work in a thread.
    /// Saves results to `res`. `searching` is `true` when the search is being done, `false`
    /// after it is over. `thumbnail_paths` is handled immediately here for optimisation purposes  
    fn get_results(
        se: Search,
        res: Arc<Mutex<Vec<Vec<u8>>>>,
        searching: Arc<Mutex<bool>>,
        db: TagMaidDatabase,
        thumbnail_paths: Arc<RwLock<HashMap<Vec<u8>, Arc<PathBuf>>>>,
    ) -> Result<()> {
        info!("Grabbing results");
        // No cached results
        let fs_db_mutex = db.get_fs_db();
        let fs_db: MutexGuard<TagDatabase> = fs_db_mutex.lock().unwrap();
        let mut cands = match se.first_tag() {
            Some(s) => fs_db.get_hashes_from_tag(&s),
            None => fs_db.get_all_file_hashes(),
        };
        drop(fs_db);
        if cands.is_err() {
            *searching.lock().unwrap() = false;
            return Err(cands.unwrap_err());
        } else {
            cands.as_mut().unwrap().retain(|hash| {
                let tags = &db.get_tags_from_hash(hash);
                match tags {
                    Ok(tags) => se.filter_post(&tags),
                    Err(..) => false,
                }
            });
            let mut pool = Vec::new();
            for i in cands.as_ref().unwrap().iter() {
                if let Ok(tf) = db.get_tagfile_from_hash(i) {
                    let thumbnail_paths = thumbnail_paths.clone();
                    pool.push(std::thread::spawn(move || {
                        Self::get_thumbnail_path(thumbnail_paths, tf.into());
                    }));
                }
                while pool.len() > 5 {
                    pool.retain(|i| !i.is_finished());
                }
            }
            for i in pool.into_iter() {
                i.join().ok();
            }
            let results_vec: Vec<Vec<u8>> = cands.unwrap().into_iter().collect();
            *res.lock().unwrap() = results_vec;
            *searching.lock().unwrap() = false;
        }
        Ok(())
    }

    /// Interactive app logo. Click on it to go back.
    fn ui_logo(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        // cute logo bit
        let logo_path = PathBuf::from("logo.png");
        let image_texture =
            &self.get_texture(ctx, &TextureLabel::FileThumbnail(Arc::new(logo_path)));
        let logo = ui.image(image_texture.id(), egui::vec2(60.0, 60.0));
        let response = &logo.interact(egui::Sense::click());
        if response.clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            match self.mode {
                ViewPage::View => self.mode = ViewPage::Results,
                ViewPage::Edit => {
                    self.edit_add_tags = String::new(); //reset UI if go back (doesn't affect tags that get added)
                    self.edit_tags = BTreeSet::new();
                    self.mode = ViewPage::Search;
                }
                _ => self.mode = ViewPage::Search,
            }
        }
    }

    /// The "Add" tab. Checks if a file can be edited, and if there is,
    /// it redirects the user to the editing tab. Otherwise it redirects
    /// to `ui_add_drag()`
    fn ui_add(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.ui_logo(ctx, ui);
            ui.label(egui::RichText::new("Add").font(egui::FontId::monospace(40.0)));
            ui.add_space(15.0);
        });
        ui.add(egui::Separator::default().horizontal());
        ui.add_space(5.0);
        match &self.add_path {
            Some(dragged_file_path) => {
                let maybe_tagfile: Option<TagFile> =
                    TagFile::initialise_from_path(dragged_file_path).ok();
                match maybe_tagfile {
                    Some(tagfile) => {
                        // Load file hash and tags (if file in database), then send to edit mode

                        // Caching file hash and tags in a BTreeSet to have it look sorted
                        self.edit_tags = BTreeSet::new();
                        let maybe_tags = &self.db.get_tags_from_hash(&tagfile.file_hash).ok();
                        self.edit_hash = Some((&tagfile.file_hash).clone());
                        match maybe_tags {
                            Some(tags) => {
                                for tag in tags {
                                    self.edit_tags.insert(tag.clone());
                                }
                            }
                            None => {
                                // File not in db so add it
                                self.db.update_tagfile(&tagfile).ok();
                            }
                        }
                        self.mode = ViewPage::Edit;
                    }
                    None => {
                        self.ui_error(
                            ctx,
                            ui,
                            "Sorry, we couldn't initialise this file you just dragged.",
                        );
                    }
                }
            }
            None => {
                self.ui_add_drag(ctx, ui);
            }
        }
    }

    /// Something that we hope the user never gets to see
    fn ui_error(&self, _ctx: &egui::Context, ui: &mut egui::Ui, error_string: &str) {
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new("Uh oh...").font(egui::FontId::monospace(40.0)));
        });
        ui.add_space(15.0);

        ui.centered_and_justified(|ui| {
            ui.label(egui::RichText::new(error_string).font(egui::FontId::monospace(20.0)));
        });
    }

    /// The tab when editing a file (adding or removing tags to it)
    fn ui_edit(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.ui_logo(ctx, ui);
            ui.label(egui::RichText::new("Edit").font(egui::FontId::monospace(40.0)));
            ui.add_space(15.0);
        });
        ui.add(egui::Separator::default().horizontal());
        ui.add_space(5.0);

        // It is assumed that the file is already in the database and was checked before calling ui_edit()
        match &self.edit_hash {
            Some(hash) => {
                let mut tagfile = self.db.get_tagfile_from_hash(hash).unwrap();
                let image_texture = &self.get_texture(
                    ctx,
                    &TextureLabel::FileThumbnail(Arc::new(PathBuf::from(tagfile.get_path()))),
                );
                let file_tags = &self.edit_tags.clone();
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.set_max_height(420.69);
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.set_min_width(220.0);
                                ui.set_max_width(220.0);
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new("Tags")
                                            .font(egui::FontId::monospace(17.0)),
                                    );
                                    ui.add_space(5.0);
                                    for tag in file_tags.clone().into_iter() {
                                        ui.horizontal(|ui| {
                                            if ui.button("-").clicked() {
                                                // TODO: If all tags are removed, safely delete file
                                                // (right now we prevent the user from doing that because it panics)
                                                if file_tags.len() > 1 {
                                                    // Update DB
                                                    tagfile.remove_tag(&tag).ok();
                                                    self.db.update_tagfile(&tagfile).ok();

                                                    // Update UI
                                                    self.edit_tags.remove(&tag);
                                                }
                                            }
                                            let _tag_label = ui.label(
                                                egui::RichText::new(&tag)
                                                    .font(egui::FontId::monospace(14.0)),
                                            );
                                        });
                                    }
                                });
                            });
                        });
                        ui.vertical(|ui| {
                            ui.set_min_width(400.0);
                            ui.set_max_width(400.0);
                            ui.set_max_height(350.0);
                            ui.centered_and_justified(|ui| {
                                let height_limit = 340.0;
                                let width_limit = 390.0;

                                let img_size: Vec2 = image_texture.size_vec2();
                                let mut scaled_height = height_limit.clone();
                                let mut scaled_width = &img_size.x * height_limit / &img_size.y;
                                if &scaled_width > &width_limit {
                                    scaled_width = width_limit.clone();
                                    scaled_height = &img_size.y * width_limit / &img_size.x;
                                }
                                ui.centered_and_justified(|ui| {
                                    ui.image(
                                        image_texture.id(),
                                        egui::vec2(scaled_width, scaled_height),
                                    );
                                });
                            });
                        });
                    });
                    ui.centered_and_justified(|ui| {
                        ui.horizontal(|ui| {
                            ui.set_max_height(30.0);
                            ui.label(
                                egui::RichText::new("Add tags:")
                                    .font(egui::FontId::monospace(17.0)),
                            );
                            let add_tag_input = ui.add(
                                egui::TextEdit::singleline(&mut self.edit_add_tags)
                                    .min_size(egui::vec2(620.0, 18.0)),
                            );
                            // THIS SUCKS ASS!!!!!
                            if !add_tag_input.has_focus() && !add_tag_input.lost_focus() {
                                add_tag_input.request_focus();
                            }
                            if (add_tag_input.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                                || ui.button("Add").clicked()
                            {
                                let split_tags = self.edit_add_tags.split_whitespace();
                                for tag in split_tags {
                                    if data::tag_util::is_tag_name_valid(&tag) {
                                        // TODO: Adding tags one by one sucks
                                        tagfile.add_tag(&tag).ok();
                                        self.db.update_tagfile(&tagfile).ok();
                                        self.edit_tags.insert(tag.to_string());
                                    }
                                }
                                self.edit_add_tags = String::new();
                            }
                        });
                    });
                });
            }
            None => {
                self.ui_error(ctx, ui, "Sorry, we couldn't find any files to edit!");
            }
        }
    }

    /// The "Add" tab when no file is being edited.
    /// It tells the user to drag a file or choose one with the file dialog.
    fn ui_add_drag(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.add_space(50.0);
        ui.horizontal(|ui| {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("Drag a file here to add it").font(egui::FontId::monospace(40.0)));
                ui.add_space(20.0);
                let click_label = ui.label(egui::RichText::new("...or click here to import it yourself").font(egui::FontId::monospace(20.0)));
                let click_label_response = click_label.interact(egui::Sense::click());
                if click_label_response.clicked() && self.path_future.is_none() {
                    self.path_future = Some(std::thread::spawn(|| rfd::FileDialog::new().pick_file()));
                }
            });
            if let Some(s) = &self.path_future {
                if s.is_finished() {
                    // the first unwrap is checked the second one carries the panic from the thread
                    let r = self.path_future.take().unwrap().join().unwrap();
                    if let Some(p) = r {
                        self.add_path = Some(p.clone());
                        self.edit_tags = BTreeSet::new();
                        let empty_tagfile = TagFile::initialise_from_path(&p).ok();
                        match empty_tagfile {
                            Some(loaded_tagfile) => {
                                let maybe_db_tagfile = self.db.get_tagfile_from_hash(&loaded_tagfile.file_hash).ok();
                                match maybe_db_tagfile {
                                    Some(tagfile) => {
                                        // File in db
                                        for tag in tagfile.get_tags().clone() {
                                            self.edit_tags.insert(tag);
                                        }
                                        self.edit_hash = Some(tagfile.file_hash.clone());
                                    }
                                    None => {
                                        // File not in db yet; add it
                                        self.db.update_tagfile(&loaded_tagfile).ok();
                                        self.edit_hash = Some(loaded_tagfile.file_hash.clone());
                                    }
                                }
                                // TODO: is this reaaaally necessary?
                                if self.db.get_tagfile_from_hash(&self.edit_hash.clone().unwrap()).is_ok() {
                                    // File is in db now, send user to edit mode
                                    self.mode = ViewPage::Edit;
                                } else {
                                    self.ui_error(ctx, ui, "Sorry, we couldn't initialise this file you just tried to add in the database.");
                                }
                            }
                            None => {
                                self.ui_error(ctx, ui, "Sorry, we couldn't initialise this file you just tried to add.");
                            }
                        }
                    }
                }
            }
        });
    }

    /// The "View" tab. Appears when viewing a file in results by clicking on it
    fn ui_view(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.ui_logo(ctx, ui);
            ui.label(egui::RichText::new("View").font(egui::FontId::monospace(40.0)));
            ui.add_space(15.0);
        });
        ui.add(egui::Separator::default().horizontal());
        ui.add_space(5.0);
        match &self.viewmode_tagfile_hash {
            Some(hash) => {
                let tagfile = self.db.get_tagfile_from_hash(&hash).unwrap();
                let image_path = tagfile.get_path().to_owned();
                let image_texture =
                    &self.get_texture(ctx, &TextureLabel::FileThumbnail(Arc::new(image_path)));

                ui.horizontal(|ui| {
                    if ui.button("Remove file").clicked() {
                        self.remove_tagfile = Some(tagfile.clone());
                        self.mode = ViewPage::RemoveFile;
                    }
                    if ui.button("Edit tags").clicked() {
                        self.edit_hash = Some(hash.clone());
                        self.edit_tags = BTreeSet::new();
                        for tag in tagfile.get_tags().clone() {
                            self.edit_tags.insert(tag);
                        }
                        self.mode = ViewPage::Edit;
                    }
                    if ui.button("Copy path").clicked() {
                        ctx.output_mut(|out| {
                            out.copied_text = tagfile.get_path().to_string_lossy().to_string();
                        });
                    }
                    if ui.button("Open file").clicked() {
                        #[cfg(target_os = "windows")]
                        {
                            std::process::Command::new("cmd")
                                .arg("/c")
                                .arg(&tagfile.path)
                                .spawn()
                                .expect("Failed to open file");
                        }
                        #[cfg(target_os = "linux")]
                        {
                            std::process::Command::new("open")
                                .arg(&tagfile.path)
                                .spawn()
                                .expect("Failed to open file");
                        }
                    }
                });

                ui.add_space(15.0);

                ui.group(|ui| {
                    let height_limit = 340.0;
                    let width_limit = 650.0;

                    let img_size: Vec2 = image_texture.size_vec2();
                    let mut scaled_height = height_limit.clone();
                    let mut scaled_width = &img_size.x * height_limit / &img_size.y;
                    if &scaled_width > &width_limit {
                        scaled_width = width_limit.clone();
                        scaled_height = &img_size.y * width_limit / &img_size.x;
                    }
                    ui.centered_and_justified(|ui| {
                        ui.image(image_texture.id(), egui::vec2(scaled_width, scaled_height));
                    });
                });
            }
            None => {
                ui.label("No file selected");
            }
        }
    }

    fn ui_remove_file(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        match &self.remove_tagfile {
            Some(tagfile) => {
                ui.add_space(5.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Are you sure you want to remove this file?")
                            .font(egui::FontId::monospace(20.0)),
                    );
                    ui.label(
                        egui::RichText::new("All the tags will be deleted")
                            .font(egui::FontId::monospace(14.0)),
                    );
                    ui.add_space(5.0);
                    ui.vertical_centered_justified(|ui| {
                        ui.columns(2, |ui| {
                            let yes_button_text = egui::RichText::new("Yes, remove")
                                .font(egui::FontId::monospace(14.0))
                                .color(egui::Color32::RED);
                            let no_button_text = egui::RichText::new("No, cancel")
                                .font(egui::FontId::monospace(14.0))
                                .color(egui::Color32::BLACK);
                            if ui[0].button(yes_button_text).clicked() {
                                // Removing all tags and calling update_tagfile()
                                // will remove it from the database
                                let mut cleared_tagfile = tagfile.clone();
                                cleared_tagfile.remove_all_tags().ok();
                                let _ = self.db.update_tagfile(&cleared_tagfile);
                                // We remove the file_hash of the deleted file from the results
                                self.results
                                    .lock()
                                    .unwrap()
                                    .retain(|hash| hash != &cleared_tagfile.file_hash);

                                // File is deleted, so we go back on results instead of view mode
                                self.mode = ViewPage::Results;
                            }
                            if ui[1].button(no_button_text).clicked() {
                                // File isn't deleted so we go back on view mode
                                self.mode = ViewPage::View;
                            }
                        });
                    });
                });
                ui.add_space(15.0);
                ui.vertical_centered(|ui| {
                    let image_texture = &self.get_texture(
                        ctx,
                        &TextureLabel::FileThumbnail(Arc::new(PathBuf::from(tagfile.get_path()))),
                    );

                    let height_limit = 340.0;
                    let width_limit = 650.0;

                    let img_size: Vec2 = image_texture.size_vec2();
                    let mut scaled_height = height_limit.clone();
                    let mut scaled_width = &img_size.x * height_limit / &img_size.y;
                    if &scaled_width > &width_limit {
                        scaled_width = width_limit.clone();
                        scaled_height = &img_size.y * width_limit / &img_size.x;
                    }
                    ui.centered_and_justified(|ui| {
                        ui.image(image_texture.id(), egui::vec2(scaled_width, scaled_height));
                    });
                });
            }
            None => {
                self.mode = ViewPage::Results;
            }
        }
    }

    /// The "Search" tab
    fn ui_search(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.ui_logo(ctx, ui);
            ui.label(egui::RichText::new("Search").font(egui::FontId::monospace(40.0)));
            ui.add_space(15.0);
        });
        ui.add(egui::Separator::default().horizontal());
        ui.add_space(5.0);
        ui.vertical_centered(|ui| {
            let search_input = ui.text_edit_singleline(&mut self.search);
            if !search_input.has_focus() && !search_input.lost_focus() {
                // Force focus on the search bar since that's all there is in the window
                // (not a very nice way of doing it though)
                search_input.request_focus();
            }
            if (search_input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                || ui.button("Search").clicked()
            {
                // Handle search

                match Search::from_string(&self.search) {
                    Ok(v) => {
                        *self.update_search.lock().unwrap() = true;
                        self.search_err = None;
                        let nbool = Arc::clone(&self.update_search);
                        let nres = Arc::clone(&self.results);
                        let db = self.db.clone();
                        let search = v.clone();
                        let thumbnail_paths = self.thumbnail_paths.clone();
                        let mut is_cached = false;
                        self.search_options = Some(v.clone());

                        // Try finding a cached search

                        match self.db.get_cache().get_search(&search) {
                            Some(search_results) => {
                                is_cached = true;
                                *nres.lock().unwrap() = search_results.clone();
                                *nbool.clone().lock().unwrap() = false;
                            },
                            None => {}
                        }

                        // Search wasn't cached
                        if !is_cached {
                            let handle = std::thread::spawn(move || {
                                match Self::get_results(
                                    search,
                                    nres.clone(),
                                    nbool,
                                    db,
                                    thumbnail_paths,
                                ) {
                                    Ok(..) => {}
                                    Err(..) => {
                                        nres.clone().lock().unwrap().clear();
                                    }
                                }
                            });
                            /**
                            So there was basically a data race or whatever its called because
                            the thread takes time and the caching functions would accidentally
                            access the previous result because the thread wasn't updating it yet

                            So... I used join(), it works perfectly as intended, but this makes
                            me question whether it defeats the whole purpose of the thread. Not that it
                            panics or whatever, but idk.

                            I couldn't implement the "put in cache" apart in the get_results() function 
                            because the cache is a RefCell and even though it compiles and works fine,
                            the "real" RefCell never gets updated so the stuff never got cached.
                            
                            Idk how to fix this other than join(). Imo given that we can only do one search
                            at once (as a user) I'd say it's fine right now. But something feels wrong. 
                            */
                            if handle.join().is_err() {
                                // Search failed, stop hanging it
                                *self.update_search.lock().unwrap() = false;
                            };

                            // Attempts to cache the search results
                            match self.db.get_cache().cache_search(v.clone(), self.results.clone().lock().unwrap().to_vec()) {
                                Ok(()) => {},
                                Err(err) => {
                                    // Fails silently because not being able to cache sometimes isn't
                                    // that big of a deal
                                    info!("WARNING: ui_search(): Couldn't open cache as mutable because it was already being borrowed: {err}");
                                }
                            }
                        }

                        // Search is done, send user to results page
                        self.mode = ViewPage::Results;
                    }
                    Err(s) => {
                        self.search_err = Some(s.to_string());
                    }
                }
            }
            if let Some(err) = &self.search_err {
                ui.colored_label(egui::Color32::from_rgb(255, 0, 0), err);
            }
        });
    }
    fn ui_file_thumbnail(
        &mut self,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        tagfile: &TagFile,
        thumbnail_path: Option<Arc<PathBuf>>,
    ) {
        let tagfile = Arc::new(tagfile);
        let image_texture = &self.get_texture(
            ctx,
            &match thumbnail_path {
                Some(s) => TextureLabel::FileThumbnail(s),
                None => TextureLabel::UiImages("file".to_owned()),
            },
        );
        let tagfile_name = &tagfile.get_file_name().to_owned();
        ui.centered_and_justified(|ui| {
            let image = ui.image(image_texture.id(), image_texture.size_vec2());
            let response = &image.interact(egui::Sense::click());
            if response.clicked() {
                // User clicked on an image to see the file details:
                // - Update selected hash used in View page
                // - Send user to view page
                // (In the future maybe, add browsing history by logging these clicks/hashes)
                self.viewmode_tagfile_hash = Some(tagfile.file_hash.to_vec());
                self.mode = ViewPage::View;
            }
        });
        // Some truly fucked up obscure code to clip text and add "..."
        // if the file name is too big and exceeds the width of the little box
        let mut job = LayoutJob::single_section(String::new(), TextFormat::default());
        job.wrap = TextWrapping {
            max_rows: 1,
            break_anywhere: true,
            overflow_character: Some('…'),
            ..Default::default()
        };
        // Since we deal with LayoutJob we have to use even more fucked up code
        // to make our string pretty and have a font
        job.append(
            tagfile_name,
            0.0,
            TextFormat {
                font_id: FontId::new(12.0, FontFamily::Monospace),
                ..Default::default()
            },
        );
        ui.label(job);
    }
    fn ui_results(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.ui_logo(ctx, ui);
            ui.label(egui::RichText::new("Results").font(egui::FontId::monospace(40.0)));
            ui.add_space(15.0);
        });
        ui.add(egui::Separator::default().horizontal());
        ui.add_space(5.0);
        if self.update_search.try_lock().map_or(false, |m| *m) {
            ui.spinner();
        } else {
            match self.results.clone().try_lock() {
                Ok(res) => {
                    if res.is_empty() {
                        ui.image(
                            &self.get_texture(ctx, &TextureLabel::UiImages("cobweb".to_owned())),
                            Vec2::new(100.0, 100.0),
                        );
                        ui.label("No results...");
                        return;
                    }
                    // Items Per Row (might be a config option later)
                    const IPR: usize = 6;
                    let chunks: Vec<_> = res.chunks(IPR).collect();
                    egui::ScrollArea::vertical().show_rows(ui, 120.0, chunks.len(), |ui, range| {
                        ui.vertical_centered(|ui| {
                            for row in &chunks[range] {
                                // ROW GUI
                                ui.horizontal(|ui| {
                                    ui.set_width(800.0);
                                    // ITEMS IN ROW GUI
                                    for r in row.iter() {
                                        if let Ok(tagfile) = self.db.get_tagfile_from_hash(r) {
                                            let tagfile = Arc::new(tagfile);
                                            ui.vertical(|ui| {
                                                ui.set_height(120.0);
                                                ui.set_min_width(120.0);

                                                // the "square" gui for each thing for the thin
                                                ui.group(|ui| {
                                                    ui.set_max_height(110.0);
                                                    ui.set_max_width(110.0);
                                                    self.ui_file_thumbnail(
                                                        ctx,
                                                        ui,
                                                        &tagfile,
                                                        Some(Self::get_thumbnail_path(
                                                            self.thumbnail_paths.clone(),
                                                            tagfile.clone(),
                                                        )),
                                                    );
                                                });
                                            });
                                        }
                                    }
                                });
                            }
                        });
                    });
                }
                Err(_) => {}
            };
        }
    }
}
impl eframe::App for TagMaid {
    fn on_close_event(&mut self) -> bool {
        self.conf.save().expect("failed to save");
        true
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            let a = &i.raw.dropped_files;
            if a.len() > 0 {
                let path = &a.first().unwrap().path;
                match path {
                    Some(path) => {
                        self.mode = ViewPage::Add;
                        self.add_path = Some(path.to_owned());
                    }
                    None => {}
                }
            }
        });
        egui::TopBottomPanel::top("pan").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("Search").clicked() {
                    self.mode = ViewPage::Search;
                }
                if ui.button("Results").clicked() {
                    self.mode = ViewPage::Results;
                }
                if ui.button("Add").clicked() {
                    // Reset file specifically if button is clicked
                    self.add_path = None;
                    self.edit_hash = None;
                    self.edit_tags = BTreeSet::new();
                    self.mode = ViewPage::Add;
                }
                #[cfg(feature = "ui_debug")]
                if ui.button("Debug").clicked() {
                    self.mode = ViewPage::Debug;
                }
            })
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.mode {
            ViewPage::View => {
                self.ui_view(ctx, ui);
            }
            ViewPage::Search => {
                self.ui_search(ctx, ui);
            }
            ViewPage::Results => {
                self.ui_results(ctx, ui);
            }
            ViewPage::Add => {
                self.ui_add(ctx, ui);
            }
            ViewPage::RemoveFile => {
                self.ui_remove_file(ctx, ui);
            }
            #[cfg(feature = "ui_debug")]
            ViewPage::Debug => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Set to light theme 🔆").clicked() {
                            ctx.set_visuals(egui::Visuals::light());
                            self.conf.theme = Theme::Ika;
                        }
                        if ui.button("Set to dark theme 🌑").clicked() {
                            ctx.set_visuals(egui::Visuals::dark());
                            self.conf.theme = Theme::Nameless;
                        }
                    });
                    ui.label("Egui settings:");
                    ctx.settings_ui(ui);
                });
            }
            ViewPage::Edit => {
                self.ui_edit(ctx, ui);
            }
        });
    }
}
