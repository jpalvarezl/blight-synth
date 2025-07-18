use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Sequencer {
    pub sequences: Vec<Sequence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Sequence {
    pub name: String,
    pub instrument_id: u8,
    pub events: Vec<SequenceEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum SequenceEvent {
    NoteOn {
        midi_value: u8,
        velocity: u8,
        start_time: u32,
    },
    NoteOff {
        midi_value: u8,
    },
    CC {
        controller: u8,
        value: u8,
        start_time: u32,
    },
}
