use crate::AudioEngineState;

#[tauri::command]
pub fn set_attack(value: f32, audio_engine_state: tauri::State<AudioEngineState>) {
    audio_engine_state.0.set_attack(value);
}

#[tauri::command]
pub fn set_decay(value: f32, audio_engine_state: tauri::State<AudioEngineState>) {
    audio_engine_state.0.set_decay(value);
}

#[tauri::command]
pub fn set_sustain(value: f32, audio_engine_state: tauri::State<AudioEngineState>) {
    audio_engine_state.0.set_sustain(value);
}

#[tauri::command]
pub fn set_release(value: f32, audio_engine_state: tauri::State<AudioEngineState>) {
    audio_engine_state.0.set_release(value);
}
