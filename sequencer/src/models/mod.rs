use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Sequencer {
    pub sequences: Vec<Pattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Pattern {
    pub name: String,
    pub instrument_id: u8,
    pub events: Vec<PatternEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[repr(C)]
pub struct  PatternEvent {
    pub note: u8,
    pub volume: u8,
    pub effect: EffectType,
    pub effect_param: u8,
}

/// Enum representing all primary effect types, based on ProTracker/XM standards.
/// Also stored as a single byte.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
#[repr(u8)]
pub enum EffectType {
    Arpeggio = 0x0, // When effect_param is 0 then this variant is actually None (no effect)
    PortamentoUp = 0x1,
    PortamentoDown = 0x2,
    TonePortamento = 0x3,
    Vibrato = 0x4,
    TonePortamentoVolumeSlide = 0x5,
    VibratoVolumeSlide = 0x6,
    Tremolo = 0x7,
    SetPanning = 0x8,
    SetSampleOffset = 0x9,
    VolumeSlide = 0xA,
    PositionJump = 0xB,
    SetVolume = 0xC,
    PatternBreak = 0xD,
    Extended = 0xE, // For E-commands like fine slides, etc.
    SetSpeedOrBPM = 0xF,
}