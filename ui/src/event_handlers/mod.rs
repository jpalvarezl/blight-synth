pub(crate) mod keyboard;

use egui::Key;
use crate::Content;

pub trait InputStateHandler {
    fn handle_input(&mut self, context: &egui::Context);
}
