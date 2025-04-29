use audio_backend::synths::oscillator::Waveform;
use egui::Ui;

use crate::Content;

pub(super) fn init(root: &mut Ui, content: &mut Content) {
    let mut selected_waveform = content.oscillator_viewmodel.get_oscillator().read().unwrap().waveform.clone();
    
    root.radio_value(&mut selected_waveform, Waveform::Sine, Waveform::Sine.to_string());
    root.radio_value(&mut selected_waveform, Waveform::Square, Waveform::Square.to_string());
    root.radio_value(&mut selected_waveform, Waveform::Saw, Waveform::Saw.to_string());
    root.radio_value(&mut selected_waveform, Waveform::Triangle, Waveform::Triangle.to_string());

    content.oscillator_viewmodel.set_waveform(selected_waveform);
}
