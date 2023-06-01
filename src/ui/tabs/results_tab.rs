use crate::TagFile;
use dioxus::prelude::*;
use std::collections::HashSet;

pub fn render<'a>(cx: &'a ScopeState) -> Element<'a> {
    let results = use_ref(cx, im_rc::Vector::<TagFile>::default);

    cx.render(rsx! {
        h1 { "Results" }
        div {
            results.read().iter().map(|id| rsx!(crate::ui::components::result_file_component::render { }))
        }
    })
}
