use crate::SynthesizerState;

#[tauri::command]
pub fn set_attack(value: f32, synthesizer_state: tauri::State<SynthesizerState>) {
    let mut synthesizer = synthesizer_state.0.lock().expect("Failed to lock synthesizer state");
    let (_, decay, sustain, release) = synthesizer.current_adsr_params();
    synthesizer.set_adsr(value, decay, sustain, release);
}

#[tauri::command]
pub fn set_decay(value: f32, synthesizer_state: tauri::State<SynthesizerState>) {
    let mut synthesizer = synthesizer_state.0.lock().expect("Failed to lock synthesizer state");
    let (attack, _, sustain, release) = synthesizer.current_adsr_params();
    synthesizer.set_adsr(attack, value, sustain, release);
}

#[tauri::command]
pub fn set_sustain(value: f32, synthesizer_state: tauri::State<SynthesizerState>) {
    let mut synthesizer = synthesizer_state.0.lock().expect("Failed to lock synthesizer state");
    let (attack, decay, _, release) = synthesizer.current_adsr_params();
    synthesizer.set_adsr(attack, decay, value, release);
}

#[tauri::command]
pub fn set_release(value: f32, synthesizer_state: tauri::State<SynthesizerState>) {
    let mut synthesizer = synthesizer_state.0.lock().expect("Failed to lock synthesizer state");
    let (attack, decay, sustain, _) = synthesizer.current_adsr_params();
    synthesizer.set_adsr(attack, decay, sustain, value);
}
