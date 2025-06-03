// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use audio_backend::synths::new_synthesizer::Synthesizer;
use std::sync::Mutex;

mod adsr;
mod synth;

// Shared state for the oscillator
pub struct SynthesizerState(pub Mutex<Synthesizer>);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Synthesizer::default() now works because we added the Default trait
    // let synthesizer = Synthesizer::new(44100.0, 8);
    let synthesizer = Synthesizer::new(24000.0, 8); // Use the default synthesizer
    // Pass the sample rate from the default synth to the audio thread setup if needed
    // Or ensure start_audio_thread uses the synth's sample rate
    audio_backend::start_audio_thread(synthesizer.clone());
    tauri::Builder::default()
        .manage(SynthesizerState(Mutex::new(synthesizer)))
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
