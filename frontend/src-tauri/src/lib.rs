// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use audio_backend::devices::audio_engine::AudioEngine;

mod adsr;
mod synth;

// Shared state for the audio engine
pub struct AudioEngineState(pub AudioEngine);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Create the lock-free audio engine (this automatically starts the audio stream)
    let audio_engine = audio_backend::start_lockfree_audio_engine(44100.0, 8)
        .expect("Failed to start audio engine");
    
    tauri::Builder::default()
        .manage(AudioEngineState(audio_engine))
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
