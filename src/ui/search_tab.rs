pub mod search_input;

pub struct SearchUIData {
    search_field_text: String,
}
impl SearchUIData {
    pub fn new() -> SearchUIData {
        SearchUIData {
            search_field_text: String::new(),
        }
    }
}
pub fn render(ui_data: &mut crate::ui::TagMaid, _ctx: &egui::Context, ui: &mut egui::Ui) {
    ui.label("Hello");
    let mut search_ui_data = SearchUIData::new();
    search_input::render(_ctx, ui);
}
