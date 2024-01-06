use super::view_models::oscillator::OscillatorViewModel;

pub(crate) mod keyboard;

pub trait InputStateHandler {
    fn handle_input(&mut self, context: &egui::Context, oscillator_viewmodel: &OscillatorViewModel);
}
