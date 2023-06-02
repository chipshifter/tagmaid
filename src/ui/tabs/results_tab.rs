use crate::TagFile;
use crate::UIData;
use crate::get_ui_data;
use dioxus::prelude::*;
use std::collections::HashSet;

pub fn render(cx: Scope) -> Element {
    let ui_data = get_ui_data(cx);
    let results = ui_data.read().get_search_results();

    let results_rendered = results.iter().map(|result| {
        let tf_option = ui_data.read().db().get_tagfile_from_hash(&result).ok();
        if tf_option.is_some() {
            let tf: TagFile = tf_option.unwrap();
            rsx!(ResultComponent { tagfile: tf })
        } else {
            rsx!(h3 { "no "})
        }
    });

    cx.render(rsx! {
        results_rendered
    })
}

#[inline_props]
fn ResultComponent(cx: Scope, tagfile: TagFile) -> Element {
    cx.render(rsx! {
        style { include_str!("../css/result_file_component.css") }
        div {
            span { "{tagfile.get_file_name()}" }
            img { src: "{tagfile.get_thumbnail_path().display()}" }
        }
    })
}

