use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// A single point in an envelope.
pub struct EnvelopePoint {
    /// Time position of the point.
    pub frame: u16,

    /// Value at this point. (e.g., 0-64 for volume)
    pub value: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// Defines a generic envelope for modulating parameters like volume or filter cutoff.
pub struct Envelope {
    pub points: Vec<EnvelopePoint>,
    pub sustain_point: u8,
    pub loop_start_point: u8,
    pub loop_end_point: u8,
    pub enabled: bool,
}
