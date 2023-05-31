use super::SearchUIData;
use crate::data::search_command::Search;

pub fn process_search(query_string: &String) {
    let parsed_search_query = Search::from_string(query_string).unwrap();
}

pub fn render(_ctx: &egui::Context, ui: &mut egui::Ui, search_ui_data: &mut SearchUIData) {
    let search_textfield = ui.text_edit_singleline(&mut search_ui_data.search_field_text);
    if search_textfield.changed() {
        let query_string = search_ui_data.search_field_text.clone();
        println!("Query changed: {}", query_string);

        if ui.input(|i| i.key_pressed(egui::Key::Space)) {
            println!("Space pressed");
        }
    }
    if search_textfield.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        let query_string = search_ui_data.search_field_text.clone();
        println!("Enter: {}", &query_string);
        process_search(&query_string)
    }
}
