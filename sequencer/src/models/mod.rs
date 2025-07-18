use bincode::{Decode, Encode};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Sequence {
    pub name: String,
    pub events: Vec<SequenceEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum SequenceEvent {
    NoteOn { midi_value: u8, velocity: u8, start_time: u32 },
    NoteOff { midi_value: u8 },
    CC { controller: u8, value: u8, start_time: u32 },
}
