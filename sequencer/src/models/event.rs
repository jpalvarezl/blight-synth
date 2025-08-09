use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::models::EffectType;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Encode, Decode)]
#[repr(C)]
pub struct Event {
    pub note: u8,
    pub volume: u8,
    pub instrument_id: u8,
    pub effect: EffectType,
    pub effect_param: u8,
}

impl Default for Event {
    fn default() -> Self {
        Event {
            note: 0,
            volume: 0,
            // effect == EffectType::Arpeggio && effect_param == 0 => no effect
            instrument_id: 0,
            effect: EffectType::Arpeggio,
            effect_param: 0,
        }
    }
}

const NO_NOTE: u8 = 0;
const NOTE_OFF: u8 = 97;
const NO_EFFECT: u8 = 0;

/// Sentinel values for notes.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NoteSentinelValues {
    NoNote = NO_NOTE,
    NoteOff = NOTE_OFF,
}

/// Sentinel values for effects.
#[repr(u8)]
pub enum EffectSentinelValues {
    NoEffect = NO_EFFECT,
}

impl EffectSentinelValues {
    pub fn is_no_effect(effect: u8) -> bool {
        effect == EffectSentinelValues::NoEffect as u8
    }
}
