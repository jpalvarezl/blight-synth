use core::harmony::note::Note;

use egui::Key;

pub struct PianoKeyboard {
    state: HashMap<Note, KeyState>,
    active_octave: u8,
}

pub enum KeyState {
    Pressed,
    Held,
    Released,
}
