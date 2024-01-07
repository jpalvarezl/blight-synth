use super::{super::view_models::oscillator::OscillatorViewModel, InputStateHandler};
use core::synths::oscillator::Waveform::{Silence, Sine};
use egui::{InputState, Key};
use harmony::note::Note;
use std::collections::HashMap;

const INITIAL_OCTAVE: u8 = 4;

pub struct PianoKeyboard {
    pub keys: HashMap<Key, Note>,
    pub active_notes: HashMap<Note, KeyState>,
    pub active_octave: u8,
}

#[derive(Debug, PartialEq)]
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

    // pub fn increase_octave(&mut self) {
    //     self.active_octave += 1;
    // }

    // pub fn decrease_octave(&mut self) {
    //     self.active_octave -= 1;
    // }

    pub fn update_keys_state(
        &mut self,
        input_state: &InputState,
        oscillator_viewmodel: &OscillatorViewModel,
    ) {
        self.keys.keys().for_each(|key| {
            let note = self.keys.get(key).unwrap();
            let key_state = match (
                input_state.key_pressed(*key),
                input_state.key_down(*key),
                input_state.key_released(*key),
            ) {
                (true, _, _) => KeyState::Pressed,
                (_, true, _) => KeyState::Held,
                (_, _, true) => KeyState::Released,
                _ => return,
            };
            if key_state == KeyState::Released {
                PianoKeyboard::silence_note(oscillator_viewmodel);
                let _ = &self.active_notes.remove(note);
            } else {
                PianoKeyboard::play_sine_wave(oscillator_viewmodel);
                let _ = &self.active_notes.insert(note.clone(), key_state);
            }
        });
    }

    pub fn pressed_keys_as_string(&self) -> String {
        let mut pressed_keys = String::new();
        for (note, key_state) in &self.active_notes {
            match key_state {
                KeyState::Held => {
                    pressed_keys.push_str(&format!("{} ", note));
                }
                _ => {}
            }
        }
        return pressed_keys;
    }

    fn play_sine_wave(oscillator_viewmodel: &OscillatorViewModel) {
        oscillator_viewmodel.set_waveform(Sine);
    }

    fn silence_note(oscillator_viewmodel: &OscillatorViewModel) {
        oscillator_viewmodel.set_waveform(Silence);
    }

    fn init_keys() -> HashMap<Key, Note> {
        let mut keys = HashMap::new();

        let notes = harmony::load_notes();
        let chromatic_scale = harmony::scales::get_notes_for_cromatic_scale_in_octave(4);
        let keyboard_keys = &[
            Key::A,
            Key::W,
            Key::S,
            Key::E,
            Key::D,
            Key::F,
            Key::T,
            Key::G,
            Key::Y,
            Key::H,
            Key::U,
            Key::J,
        ];

        for (index, note) in chromatic_scale.iter().enumerate() {
            keys.insert(
                keyboard_keys[index],
                notes
                    .get(note)
                    .expect(&format!("Could not find note {}", note))
                    .clone(),
            );
        }
        println!("{:#?}", keys);
        return keys;
    }
}

impl InputStateHandler for PianoKeyboard {
    fn handle_input(
        &mut self,
        context: &egui::Context,
        oscillator_viewmodel: &OscillatorViewModel,
    ) {
        context.input(|input_state| self.update_keys_state(input_state, oscillator_viewmodel));
    }
}
