pub(crate) mod keyboard;

pub trait InputStateHandler {
    fn handle_input(&mut self, context: &egui::Context);
}
