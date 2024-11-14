use dioxus::prelude::*;
use dioxus::html::HasFileData;
use futures_util::StreamExt;
use std::{fmt::Debug, path::PathBuf};

use crate::{data::tag_file::TagFile, database::tagmaid_database::get_database_path, get_ui_data, ui::get_tagmaiddatabase};

#[derive(Clone, Copy)]
enum PreviewType {
    None,
    Image,
    Video
}

pub fn render() -> Element {
    let mut path: Signal<Option<PathBuf>> = use_signal(|| None);
    let mut disabled: Signal<bool> = use_signal(|| false);
    let mut added_via_drop: Signal<bool> = use_signal(|| false);
    let mut file_name: Signal<Option<String>> = use_signal(|| None);
    let mut old_tags: Signal<Vec<String>> = use_signal(Vec::new);
    let mut new_tags: Signal<Vec<String>> = use_signal(Vec::new);
    let mut tag_draft = use_signal(String::new);
    let mut valid_tag = use_signal(|| true);
    let mut preview = use_signal(|| PreviewType::None);
    let mut note_draft = use_signal(|| None);
    let mut transcript_draft = use_signal(|| None);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let db = get_tagmaiddatabase();
    let db2 = db.clone();
    let file_picker_routine = use_coroutine( |mut chan: UnboundedReceiver<bool>| async move {
        loop {
            if chan.next().await.unwrap_or(false) {
                let file_handle = rfd::AsyncFileDialog::new().pick_file().await;
                disabled.set(false);
                if let Some(file_handle) = file_handle {
                    let old_path = path.read().clone();
                    if let Some(old_path) = old_path {
                        if *added_via_drop.read() {
                            std::fs::remove_file(old_path).ok();
                            added_via_drop.set(false);
                        }
                    }
                    path.set(Some(file_handle.path().to_owned()));
                    file_name.set(Some(file_handle.file_name()));
                }
            }
        }
    });
    
    const DROP_AREA_CSS: &str = "width: 250pt; height: 250pt; border: dotted 2pt; border-radius 3pt; align-content: center; text-align: center;";

    rsx! {
        h1 { "Add" },
        form {
            input {
                r#type: "button",
                value: "Select File",
                disabled: disabled,
                onclick: move |_event| {
                    file_picker_routine.send(true);
                    disabled.set(true);
                }
            },
        },
        {if path.read().is_some() {
            match *preview.read() {
                PreviewType::None => {
                    rsx!{div {}}
                }
                PreviewType::Image => {
                    rsx!{
                        img {
                            src: path.read().as_ref().unwrap().to_str().unwrap_or_default()
                        }
                    }
                }
                PreviewType::Video => {
                    rsx!{
                        video {
                            src: path.read().as_ref().unwrap().to_str().unwrap_or_default()
                        }
                    }
                }
            }
        } else {
            rsx!{div {}}
        }}
        select {
            oninput: move |ev| {
                match ev.value().as_ref() {
                    "no-preview" => {preview.set(PreviewType::None);}
                    "image" => {preview.set(PreviewType::Image);}
                    "video" => {preview.set(PreviewType::Video);}
                    _ => panic!("Unexpected value from combobox")
                }
            },
            option {value: "no-preview", "No Preview" }
            option {value: "image", "Image"}
            option {value: "video", "Video"}
        }
        div {
            style: DROP_AREA_CSS,
            ondrop: move |event| async move {
                if let Some(files) = event.files() {
                    if let Some(file_name) = files.files().first() {
                        if let Some(file_data) = files.read_file(file_name).await {
                            // Update this when configs are added
                            let mut file_path = get_database_path(None).unwrap();
                            file_path.push(file_name);
                            if let Some(old_path) = path.read().clone() {
                                if *added_via_drop.read() {
                                    std::fs::remove_file(old_path).ok();
                                }
                            }
                            match std::fs::write(&file_path, file_data) {
                                Ok(()) => {
                                    path.set(Some(file_path));
                                    added_via_drop.set(true);
                                },
                                Err(e) => {
                                    error.set(Some(format!("Error dropping off file {}", e)));
                                },
                            }

                        }
                    }
                }
            },
            "Drop your file here"
        },
        {error.read().iter().map(|it| {
            rsx! {
                p {
                    style: "color: red;",
                    {it.as_str()}
                },
                button {
                    style: "border: none; background: inherit;",
                    onclick: move |_| {
                        error.set(None);
                    },
                    "×"
                }
            }
        })},
        p {
            {
                if let Some(path) = path.read().as_ref() {
                    format!("{}", path.to_string_lossy())
                } else {
                    String::from("No file selected")
                }
            }
        },

        form {
            onsubmit: move |_| {
                if crate::data::tag_util::is_tag_name_valid(tag_draft.read().as_ref()) {
                    match db.get_tag_info(&tag_draft.read()) {
                        Some(_tag_info) => {
                            old_tags.push(tag_draft.read().clone());
                        }
                        None => {
                            new_tags.push(tag_draft.read().clone());
                        }
                    }
                    tag_draft.write().clear()
                } else {
                    valid_tag.set(true);
                }
            },
            label {
                r#for: "tag-field",
                "Add Tag:"
            },
            input {
                id: "tag-field",
                r#type: "text",
                value: "{tag_draft.read()}",
                oninput: move |event| {
                    if *valid_tag.read() {valid_tag.set(true);}
                    tag_draft.set(event.value())
                },
            },
            {
                if !*valid_tag.read() {
                    rsx!{
                        span {
                            style: "color: red;",
                            "Invalid tag name"
                        }
                    }
                } else {
                    None
                }
            }
        },
        h3 {"Tags"},
        div {
            TagList {list: old_tags, empty_text: "No previously used tags are attatched to this file"}
        }
        h4 {"New Tags"}
        div {
            TagList {list: new_tags, empty_text: "No new tags are attatched to this file"}
        }
        ToggleTextArea {text_area: note_draft, label: "Notes"}
        ToggleTextArea {text_area: transcript_draft, label: "Transcript"}
        button {
            disabled: path.read().is_none(),
            onclick: move |_ev| {
                let mut clear = false;
                match TagFile::initialise_from_path(&path.read().as_ref().unwrap()) {
                    Ok(mut tag_file) => {
                        for tag in old_tags.read().iter() {
                            tag_file.add_tag(tag).ok();
                        }
                        for tag in new_tags.read().iter() {
                            tag_file.add_tag(tag).ok();
                        }
                        tag_file.notes = note_draft.read().clone();
                        tag_file.transcript = transcript_draft.read().clone();

                        if let Err(e) = db2.update_tagfile(&tag_file) {
                            error.set(Some(format!("Failed to add tagged file to database: {:?}", e)))
                        } else {
                            // Reset the entire state after submitting the file
                            clear = true;
                        }
                    }
                    Err(err) => {
                        error.set(Some(format!("Failed to make tagged file: {:?}", err)));
                    }
                }
                if clear {
                    path.set(None);
                    disabled.set(false);
                    added_via_drop.set(false);
                    file_name.set(None);
                    new_tags.write().clear();
                    old_tags.write().clear();
                    tag_draft.write().clear();
                    valid_tag.set(true);
                    preview.set(PreviewType::None);
                    note_draft.set(None);
                    transcript_draft.set(None);
                    error.set(None);
                }
            },
            "Add File"
        }
    }
}

#[component]
fn ToggleTextArea(text_area: Signal<Option<String>>, label: &'static str) -> Element {
    rsx!{
        div {
            form {
                label {
                    r#for: label,
                    {label}
                }
                input {
                    r#type: "checkbox",
                    id: label,
                    checked: text_area.read().is_some(),
                    oninput: move |_ev| {
                        if text_area.read().is_some() {
                            text_area.set(None);
                        } else {
                            text_area.set(Some(String::new()));
                        }
                    },
                }
                {
                    if let Some(text) = text_area.read().as_ref() {
                        rsx!(
                            textarea {
                                style: "display: block;",
                                rows: 4,
                                cols: 25,
                                oninput: move |ev| {
                                    text_area.set(Some(ev.value()));
                                },
                                {<String as AsRef<str>>::as_ref(text)}
                            }
                        )
                    } else {
                        rsx!()
                    }
    
                }
            }
        }
    }
}

#[component]
fn TagList(list: Signal<Vec<String>>, empty_text: &'static str) -> Element {
    rsx!{
        {
            let length = list.read().len();
                if length > 0 {
                    (0..length).map(|ind| {
                        rsx! {
                            TagText {
                                list: list,
                                index: ind
                            }
                        }
                    }).into_dyn_node()
                } else {
                    rsx!(
                        i {
                            {empty_text}
                        }
                    ).into_dyn_node()
                }
        }
    }
}

#[component]
fn TagText(list: Signal<Vec<String>>, index: usize) -> Element {
    const TAG_CSS: &str = "border: grey solid 1pt;border-radius: 5pt;padding: 1pt; display: inline-flex; align-content: center;";
    const TAG_X_BTNCSS: &str = "background: inherit; border: lightgray solid 1pt; height: 17pt; width: 17pt; border-radius: 20pt; margin-left: 1pt; padding: 0;";
    rsx!{
        span {
            style: TAG_CSS,
            {(list.read()[index]).as_str()},
            button {
                style: TAG_X_BTNCSS,
                onclick: move |_| {
                    list.write().remove(index);
                },
                "×"
            }
        }
    }
}
