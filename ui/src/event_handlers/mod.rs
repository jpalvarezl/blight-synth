pub(crate) mod keyboard;

use egui::Key;
use crate::Content;

pub fn handle_input(context: &egui::Context, content: &mut Content) {
    if context.input(|i| i.key_pressed(Key::A)) {
        content.text.push_str("\nPressed");
    }
    if context.input(|i| i.key_down(Key::A)) {
        content.text.push_str("\nHeld");
    }
    if context.input(|i| i.key_released(Key::A)) {
        content.text.push_str("\nReleased");
    }
}
