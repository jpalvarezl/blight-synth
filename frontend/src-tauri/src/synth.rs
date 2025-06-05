use audio_backend::synths::waveform::Waveform;
use harmony::note;
use tauri::State;

use super::AudioEngineState;

#[tauri::command]
pub fn set_waveform(waveform: String, audio_engine_state: State<AudioEngineState>) {
    let new_waveform = match waveform.as_str() {
        "Sine" => Waveform::Sine,
        "Square" => Waveform::Square,
        "Sawtooth" => Waveform::Sawtooth,
        "Triangle" => Waveform::Triangle,
        _ => Waveform::Sine,
    };
    audio_engine_state.0.set_waveform(new_waveform);
}

#[tauri::command]
pub fn play_midi_note(midi_value: u8, audio_engine_state: State<AudioEngineState>) {
    let velocity = 1.0;
    println!("Playing MIDI note: {}", midi_value);
    audio_engine_state.0.note_on(midi_value, velocity);
}

#[tauri::command]
pub fn stop_midi_note(midi_value: u8, audio_engine_state: State<AudioEngineState>) {
    audio_engine_state.0.note_off(midi_value);
}
