use crate::SynthesizerState;

#[tauri::command]
pub fn set_attack(value: f32, synthesizer_state: tauri::State<SynthesizerState>) {
    let synthesizer = synthesizer_state.0.clone();
    synthesizer.set_attack(value);
}

#[tauri::command]
pub fn set_decay(value: f32, synthesizer_state: tauri::State<SynthesizerState>) {
    let synthesizer = synthesizer_state.0.clone();
    synthesizer.set_decay(value);
}

#[tauri::command]
pub fn set_sustain(value: f32, synthesizer_state: tauri::State<SynthesizerState>) {
    let synthesizer = synthesizer_state.0.clone();
    synthesizer.set_sustain(value);
}

#[tauri::command]
pub fn set_release(value: f32, synthesizer_state: tauri::State<SynthesizerState>) {
    let synthesizer = synthesizer_state.0.clone();
    synthesizer.set_release(value);
}
