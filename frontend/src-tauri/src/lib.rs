// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::Arc;
use audio_backend::synths::synthesizer::{Synthesizer, ActiveWaveform};
use harmony::note;
use tauri::State;

// Shared state for the oscillator
pub struct SynthesizerState(pub Arc<Synthesizer>);

#[tauri::command]
fn set_waveform(waveform: String, synthesizer_state: State<SynthesizerState>) {
    let osc = synthesizer_state.0.clone();
    let new_waveform = match waveform.as_str() {
        "Sine" => ActiveWaveform::Sine,
        "Square" => ActiveWaveform::Square,
        "Saw" => ActiveWaveform::Saw,
        "Triangle" => ActiveWaveform::Triangle,
        "Silence" => ActiveWaveform::Silence,
        _ => ActiveWaveform::Sine,
    };
    osc.set_waveform(new_waveform);
}

#[tauri::command]
fn play_midi_note(midi_value: u8, synthesizer_state: State<SynthesizerState>) {
    let freq = note::midi_to_frequency(midi_value);
    let synthesizer = synthesizer_state.0.clone();
    synthesizer.set_amplitude(0.5); // need to wire to a correct gain controller
    synthesizer.set_frequency(freq);
}

#[tauri::command]
fn stop_midi_note(synthesizer_state: State<SynthesizerState>) {
    let synthesizer = synthesizer_state.0.clone();
    synthesizer.set_frequency(0.0);
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let synthesizer = Arc::new(Synthesizer::default());
    audio_backend::start_audio_thread(synthesizer.clone());
    tauri::Builder::default()
        .manage(SynthesizerState(synthesizer))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, play_midi_note, stop_midi_note, set_waveform])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
