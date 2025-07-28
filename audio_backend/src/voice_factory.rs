use crate::{
    id::{Instrument, InstrumentId, VoiceId},
    synth_infra::{Voice, VoiceTrait},
    synths::{OscillatorNode, SamplePlayerNode},
    Envelope,
};

/// A factory for creating `Voice` objects in the non-real-time world.
/// This ensures all allocations happen before a voice is sent to the audio thread.
pub struct VoiceFactory {
    sample_rate: f32,
    // Could also hold references to instrument definitions, etc.
}

impl VoiceFactory {
    pub fn new(sample_rate: f32) -> Self {
        Self { sample_rate }
    }

    /// Creates a new, fully initialized voice, ready to be sent to the audio thread.
    pub fn create_voice(
        &self,
        voice_id: VoiceId,
        instrument: &Instrument,
        pan: f32,
    ) -> Box<dyn VoiceTrait> {
        // This is where the logic to choose a `SynthNode` based on the
        // instrument definition from the `sequencer` crate would live.
        // All `Box::new` calls happen here, safely in the NRT world.
        let envelope = Envelope::new(self.sample_rate);
        
        match instrument {
            Instrument::Oscillator => {
                let voice = Voice::new(voice_id, OscillatorNode::new(), envelope, pan);
                Box::new(voice)
            },
            Instrument::SamplePlayer => {
                let voice = Voice::new(voice_id, SamplePlayerNode::new(), envelope, pan);
                Box::new(voice)
            },
        }
    }

    pub fn create_voice_with_envelope(
        &self,
        voice_id: VoiceId,
        instrument: &Instrument,
        pan: f32,
        attack_s: f32,
        decay_s: f32,
        sustain: f32,
        release_s: f32,
    ) -> Box<dyn VoiceTrait> {
        let envelope = self.create_envelope(attack_s, decay_s, sustain, release_s);

        match instrument {
            Instrument::Oscillator => {
                let voice = Voice::new(voice_id, OscillatorNode::new(), envelope, pan);
                Box::new(voice)
            },
            Instrument::SamplePlayer => {
                let voice = Voice::new(voice_id, SamplePlayerNode::new(), envelope, pan);
                Box::new(voice)
            },
        }
    }

    pub fn create_envelope(
        &self,
        attack_s: f32,
        decay_s: f32,
        sustain: f32,
        release_s: f32,
    ) -> Envelope {
        let mut envelope = Envelope::new(self.sample_rate);
        envelope.set_parameters(attack_s, decay_s, sustain, release_s);
        envelope
    }
}
