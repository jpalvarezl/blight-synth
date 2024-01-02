use egui::{Ui, ScrollArea};
use crate::Content;

pub fn init_ui(root: &mut Ui, content: &mut Content) {
    root.heading(&content.default_output_device);
    root.heading("Press/Hold/Release example. Press A to test.");
    
    if root.button("Clear").clicked() {
        content.text.clear();
    }
    ScrollArea::vertical()
        .auto_shrink(false)
        .stick_to_bottom(true)
        .show(root, |ui| {
            ui.label(&content.text);
        });
}
