use crate::Content;
use egui::{ScrollArea, Ui};

mod wave_form_radio_button;

pub fn init_ui(root: &mut Ui, content: &mut Content) {
    root.heading(format!(
        "Default output device: '{}'",
        &content.default_output_device
    ));
    root.heading("Keys: 'A', 'W', 'S', 'E', 'D', 'F', 'T', 'G', 'Y', 'H', 'U', 'J' can be used to play notes.");
    wave_form_radio_button::init(root, content);

    ScrollArea::vertical()
        .auto_shrink(false)
        .stick_to_bottom(true)
        .show(root, |ui| {
            ui.label(&content.text);
        });
}
