use audio_backend::synths::waveform::Waveform;
use harmony::note;
use tauri::State;

use super::SynthesizerState;

#[tauri::command]
pub fn set_waveform(waveform: String, synthesizer_state: State<SynthesizerState>) {
    let mut synth = synthesizer_state.0.lock().expect("Failed to lock synthesizer state");
    let new_waveform = match waveform.as_str() {
        "Sine" => Waveform::Sine,
        "Square" => Waveform::Square,
        "Sawtooth" => Waveform::Sawtooth,
        "Triangle" => Waveform::Triangle,
        _ => Waveform::Sine,
    };
    synth.set_waveform(new_waveform);
}

#[tauri::command]
pub fn play_midi_note(midi_value: u8, synthesizer_state: State<SynthesizerState>) {
    let velocity = 1.0;
    let mut synthesizer = synthesizer_state.0.lock().expect("Failed to lock synthesizer state");
    synthesizer.note_on(midi_value, velocity);
}

#[tauri::command]
pub fn stop_midi_note(midi_value: u8, synthesizer_state: State<SynthesizerState>) {
    let mut synthesizer = synthesizer_state.0.lock().expect("Failed to lock synthesizer state");
    synthesizer.note_off(midi_value);
}
