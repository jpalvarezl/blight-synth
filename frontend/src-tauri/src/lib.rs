// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::{Arc, RwLock};
use audio_backend::{self, synths::oscillator::{Oscillator, Waveform}};
use harmony::note;
use tauri::State;

// Shared state for the oscillator
pub struct OscillatorState(pub Arc<RwLock<Oscillator>>);

#[tauri::command]
fn play_midi_note(midi_value: u8, oscillator_state: State<OscillatorState>) {
    let freq = note::midi_to_frequency(midi_value);
    let mut osc = oscillator_state.0.write().unwrap();
    osc.frequency_hz = freq;
    osc.waveform = Waveform::Sine; // or allow selection
}

#[tauri::command]
fn stop_midi_note(oscillator_state: State<OscillatorState>) {
    let mut osc = oscillator_state.0.write().unwrap();
    osc.frequency_hz = 0.0;
    osc.waveform = Waveform::Silence;
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let oscillator = Arc::new(RwLock::new(Oscillator {
        waveform: Waveform::Sine,
        frequency_hz: 0.0,
    }));
    audio_backend::start_audio_thread(oscillator.clone());
    tauri::Builder::default()
        .manage(OscillatorState(oscillator))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, play_midi_note, stop_midi_note])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
