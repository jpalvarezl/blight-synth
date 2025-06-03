// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use audio_backend::synths::new_synthesizer::Synthesizer;
use std::sync::{Arc, Mutex};

mod adsr;
mod synth;

// Shared state for the oscillator
pub struct SynthesizerState(pub Arc<Mutex<Synthesizer>>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create a single synthesizer instance wrapped in Arc<Mutex<>>
    let synthesizer = Arc::new(Mutex::new(Synthesizer::new(44100.0, 8)));
    
    // Pass the same Arc reference to the audio thread
    audio_backend::start_audio_thread(synthesizer.clone());
    
    tauri::Builder::default()
        .manage(SynthesizerState(synthesizer))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(
            tauri::generate_handler![
                synth::play_midi_note, 
                synth::stop_midi_note, 
                synth::set_waveform,
                adsr::set_attack,
                adsr::set_decay,
                adsr::set_sustain,
                adsr::set_release])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
