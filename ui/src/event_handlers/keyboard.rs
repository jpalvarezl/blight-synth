use core::harmony::note::{Note, Pitch, Accidental};
use std::collections::{HashSet, HashMap};
use egui::Key;

const INITIAL_OCTAVE: u8 = 4;

pub struct PianoKeyboard {
    pub keys: HashMap<Key, Note>,
    pub active_notes: HashMap<Note, KeyState>,
    pub active_octave: u8,
}

pub enum KeyState {
    Pressed,
    Held,
    Released,
}

impl PianoKeyboard {

    pub fn initialize() -> PianoKeyboard {
        PianoKeyboard {
            keys: PianoKeyboard::init_keys(),
            active_notes: HashMap::new(),
            active_octave: INITIAL_OCTAVE,
        }
    }

    pub fn increase_octave(&mut self) {
        self.active_octave += 1;
    }

    pub fn decrease_octave(&mut self) {
        self.active_octave -= 1;
    }

    pub fn update_key_state(&mut self, key: Key, key_state: KeyState) {
        // TODO: Handle key state changes
    }

    fn init_keys() -> HashMap<Key, Note> {
        let mut keys = HashMap::new();
        
        keys.insert(Key::A, Note { pitch: Pitch::C, accidental: Accidental::Natural, octave: INITIAL_OCTAVE });
        keys.insert(Key::W, Note { pitch: Pitch::C, accidental: Accidental::Sharp, octave: INITIAL_OCTAVE });
        keys.insert(Key::S, Note { pitch: Pitch::D, accidental: Accidental::Natural, octave: INITIAL_OCTAVE });
        keys.insert(Key::E, Note { pitch: Pitch::D, accidental: Accidental::Sharp, octave: INITIAL_OCTAVE });
        keys.insert(Key::D, Note { pitch: Pitch::E, accidental: Accidental::Natural, octave: INITIAL_OCTAVE });
        keys.insert(Key::F, Note { pitch: Pitch::F, accidental: Accidental::Natural, octave: INITIAL_OCTAVE });
        keys.insert(Key::T, Note { pitch: Pitch::F, accidental: Accidental::Sharp, octave: INITIAL_OCTAVE });
        keys.insert(Key::G, Note { pitch: Pitch::G, accidental: Accidental::Natural, octave: INITIAL_OCTAVE });
        keys.insert(Key::Y, Note { pitch: Pitch::G, accidental: Accidental::Sharp, octave: INITIAL_OCTAVE });
        keys.insert(Key::H, Note { pitch: Pitch::A, accidental: Accidental::Natural, octave: INITIAL_OCTAVE });
        keys.insert(Key::U, Note { pitch: Pitch::A, accidental: Accidental::Sharp, octave: INITIAL_OCTAVE });
        keys.insert(Key::J, Note { pitch: Pitch::B, accidental: Accidental::Natural, octave: INITIAL_OCTAVE });

        return keys;
    }
}
