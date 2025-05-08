use audio_backend::synths::synthesizer::ActiveWaveform;
use harmony::note;
use tauri::State;

use super::SynthesizerState;

#[tauri::command]
pub fn set_waveform(waveform: String, synthesizer_state: State<SynthesizerState>) {
    let osc = synthesizer_state.0.clone();
    let new_waveform = match waveform.as_str() {
        "Sine" => ActiveWaveform::Sine,
        "Square" => ActiveWaveform::Square,
        "Saw" => ActiveWaveform::Saw,
        "Triangle" => ActiveWaveform::Triangle,
        _ => ActiveWaveform::Sine,
    };
    osc.set_waveform(new_waveform);
}

#[tauri::command]
pub fn play_midi_note(midi_value: u8, synthesizer_state: State<SynthesizerState>) {
    let freq = note::midi_to_frequency(midi_value);
    let synthesizer = synthesizer_state.0.clone();
    // Use note_on instead of set_frequency
    synthesizer.note_on(freq);
}

#[tauri::command]
pub fn stop_midi_note(synthesizer_state: State<SynthesizerState>) {
    let synthesizer = synthesizer_state.0.clone();
    // Use note_off instead of set_frequency(0.0)
    synthesizer.note_off();
}
