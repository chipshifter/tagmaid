use dioxus::prelude::*;
use crate::TagFile;
use std::collections::HashSet;
use crate::ui::ui_new::UIData;

pub fn render<'a>(cx: &'a ScopeState, ui_data: &'a UIData) -> Element<'a> {
    let results = use_ref(cx, im_rc::Vector::<TagFile>::default);

    cx.render(rsx! {
        h1 { "Results" }
        div {
            results.read().iter().map(|id| rsx!(crate::ui::components::resultFileComponent::render { }))
        }
    })
}
