use super::SearchUIData;

pub fn render(_ctx: &egui::Context, ui: &mut egui::Ui, search_ui_data: &mut SearchUIData) {
    let mut query_string = String::new();
    let search_textfield = ui.text_edit_singleline(&mut search_ui_data.search_field_text);
    if (search_textfield.changed()) {
        query_string = search_ui_data.search_field_text.clone();
        println!("Query changed: {}", query_string);

        if(ui.input(|i| i.key_pressed(egui::Key::Space))) {
            println!("Space pressed");
        }
    }
    if search_textfield.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        query_string = search_ui_data.search_field_text.clone();
        println!("Enter: {}", query_string);
    }
}
