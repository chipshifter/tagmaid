use dioxus::{html::input_data::keyboard_types::Key, prelude::*};

pub fn render(cx: Scope) -> Element {
    let draft = use_ref(cx, String::new);

    cx.render(rsx! {
        h1 { "Search" }
        input {
            autofocus: "true",
            value: "{draft.read()}",
            oninput: move |event| draft.set(event.value.clone()),
            onkeydown: move |event| {
                if event.key() == Key::Enter && !draft.read().is_empty() {
                    // Do search
                    println!("Query entered: {}", &draft.read());
                }
            }
        }
    })
}
