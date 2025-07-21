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

#[serde_as] // needs to preceede #[derive]
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
/// An enum to hold the specific data for an instrument, allowing for multiple
/// types of sound generation.
pub enum InstrumentData {
    Sample(SampleParams),
    Synth(SynthParams),
    // This can be extended in the future, e.g., for FM synthesis.
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// The main instrument structure.
pub struct Instrument {
    pub name: String,
    pub data: InstrumentData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
/// Raw audio data for a single sample.
pub struct SampleData {
    pub name: String,
    pub data: Vec<i8>,
    //... other metadata
}
