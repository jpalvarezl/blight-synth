use core::harmony::note::{Note, Pitch, Accidental};
use std::collections::{HashSet, HashMap};
use egui::{Key, InputState};
use crate::Content;
use super::InputStateHandler;

const INITIAL_OCTAVE: u8 = 4;

pub struct PianoKeyboard {
    pub keys: HashMap<Key, Note>,
    pub active_notes: HashMap<Note, KeyState>,
    pub active_octave: u8,
}

#[derive(Debug)]
pub enum KeyState {
    Pressed,
    Held,
    Released,
}

impl PianoKeyboard {

    pub fn initialize() -> Self {
        Self {
            keys: Self::init_keys(),
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

    pub fn update_keys_state(&mut self, input_state: &InputState) {
        self.keys.keys().for_each(|key| {
            let note = self.keys.get(key).unwrap();
            let key_state = match (input_state.key_pressed(*key), input_state.key_down(*key), input_state.key_released(*key)) {
                (true, _, _) => KeyState::Pressed,
                (_, true, _) => KeyState::Held,
                (_, _, true) => KeyState::Released,
                _ => return,
            };
            println!("Updating key: {:#?}: {:#?}", &note, &key_state);
            &self.active_notes.insert(note.clone(), key_state);
        });
    }

    pub fn pressed_keys_as_string(&self) -> String {
        let mut pressed_keys = String::new();
        for (note, key_state) in &self.active_notes {
            match key_state {
                KeyState::Pressed => {
                    pressed_keys.push_str(&format!("{} ", note));
                },
                KeyState::Held | KeyState::Released => {},
            }
        }
        return pressed_keys;
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

impl InputStateHandler for PianoKeyboard {
    fn handle_input(&mut self, context: &egui::Context) {
        context.input(|input_state| self.update_keys_state(input_state));
        // if context.input(|i| i.key_pressed(Key::A)) {
        //     // content.text.push_str("\nPressed");
        // }
        // if context.input(|i| i.key_down(Key::A)) {
        //     // content.text.push_str("\nHeld");
        // }
        // if context.input(|i| i.key_released(Key::A)) {
        //     // content.text.push_str("\nReleased");
        // }
    }
}
