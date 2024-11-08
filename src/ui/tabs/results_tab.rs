use crate::get_ui_data;
use crate::ui::get_tagmaiddatabase;
use crate::TagFile;
use crate::UIData;
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use std::collections::HashSet;

pub fn render() -> Element {
    let ui_data = get_ui_data();
    let results = ui_data.get_search_results();
    let results_rendered = results.iter().map(|result| {
        let tf_option = get_tagmaiddatabase()
            .get_tagfile_from_hash(&result)
            .ok();
        rsx!(result_div_component {
            tagfile: tf_option.unwrap_or(TagFile::new())
        })
    });

    rsx! {
        style { {include_str!("../css/result_file_component.css")} }
        div {
            class: "result_page",
            {results_rendered}
        }
    }
}

#[component]
fn result_div_component(tagfile: TagFile) -> Element {
    if tagfile.is_empty() {
        return None;
    }
    rsx! {
        div {
            class: "result",
            img {
                src: "{tagfile.get_thumbnail_path().display()}"
            }
            hr {}
            span { "{tagfile.get_file_name()}" }
        }
    }
}
