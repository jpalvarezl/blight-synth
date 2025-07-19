use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::models::EffectType;

pub const MAX_CHANNELS: usize = 32;
pub const DEFAULT_PATTERN_ROWS: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Sequencer {
    pub sequences: Vec<Pattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// Represents a single pattern, the core building block of a song.
pub struct Pattern {
    pub num_rows: usize,
    /// Events are stored in a flat vector in row-major order.
    /// Access via `events`.
    /// This dense representation is for performance during playback.
    pub events: Vec<PatternEvent>,
}

impl Pattern {
    pub fn new(rows: usize) -> Self {
        Pattern {
            num_rows: rows,
            events: vec![],
        }
    }

    pub fn from_events(events: Vec<PatternEvent>) -> Self {
        Pattern {
            num_rows: DEFAULT_PATTERN_ROWS,
            events,
        }
    }
}

impl Default for Pattern {
    fn default() -> Self {
        Pattern::new(DEFAULT_PATTERN_ROWS)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[repr(C)]
pub struct PatternEvent {
    pub note: u8,
    pub volume: u8,
    pub effect: EffectType,
    pub effect_param: u8,
}
