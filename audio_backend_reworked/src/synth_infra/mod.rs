mod synthesizer;
mod synth_node;
mod voice; 

pub use synthesizer::*;
pub use synth_node::*;
pub use voice::*;

pub type VoiceId = u64;
pub type SampleId = u64;
pub type InstrumentId = u64;
pub type EffectChainId = u64;