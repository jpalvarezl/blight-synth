use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::models::Envelope;

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// Parameters for a simple procedural subtractive synthesizer.
pub struct SynthParams {
    // Define parameters for oscillators, filters, LFOs, etc.
    pub amp_envelope: Envelope,
    pub filter_envelope: Envelope,
    //... other synth parameters
}

#[serde_as] // needs to precede #[derive]
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// Parameters for a classic sample-based instrument.
pub struct SampleParams {
    /// Mapping of notes to sample indices in the song's sample bank.
    #[serde_as(as = "[_; 96]")]
    pub note_to_sample_map: [u8; 96],
    pub volume_envelope: Envelope,
    pub panning_envelope: Envelope,
    //... other metadata
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// Parameters for a simple oscillator instrument.
pub struct SimpleOscillatorParams {
    pub waveform: Waveform,
    pub audio_effects: Vec<AudioEffect>
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Encode, Decode, PartialEq, Eq)]
/// Waveform types for the simple oscillator.
pub enum Waveform {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    NesTriangle,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// An enum to hold the specific data for an instrument, allowing for multiple
/// types of sound generation.
pub enum InstrumentData {
    Sample(SampleParams),
    Synth(SynthParams),
    SimpleOscillator(SimpleOscillatorParams),
    // This can be extended in the future, e.g., for FM synthesis.
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// The main instrument structure.
pub struct Instrument {
    /// This ID for now will match the Track number using this Instrument.
    pub id: usize,
    pub name: String,
    pub data: InstrumentData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// An enum to hold the raw PCM data for a sample, allowing for multiple bit depths.
pub enum SampleEncoding {
    Signed8(Vec<i8>),
    Signed16(Vec<i16>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// Raw audio data for a single sample.
pub struct SampleData {
    pub name: String,
    pub data: SampleEncoding,
    pub sample_rate: u32,
    pub loop_start: u32,
    pub loop_length: u32,
    pub volume: u8,
    pub panning: u8,
    //... other metadata
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum AudioEffect {
    Reverb { room_size: f32, damping: f32 },
    Delay { time: f32, feedback: f32 }
//     Distortion { gain: f32, mix: f32 },
//     Chorus { depth: f32, rate: f32 },
}