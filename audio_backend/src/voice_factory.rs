use crate::{
    synth_infra::{Voice, VoiceTrait},
    synths::oscillator_node::OscillatorNode,
    Envelope, InstrumentId, SynthNode, VoiceId,
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
        instrument_id: InstrumentId,
        pan: f32,
    ) -> Box<dyn VoiceTrait> {
        // This is where the logic to choose a `SynthNode` based on the
        // instrument definition from the `sequencer` crate would live.
        // All `Box::new` calls happen here, safely in the NRT world.
        let synth_node = self.create_synth_node(instrument_id);

        let envelope = Envelope::new(self.sample_rate);
        let voice = Voice::new(voice_id, synth_node, envelope, pan);

        Box::new(voice)
    }

    pub fn create_voice_with_envelope(
        &self,
        voice_id: VoiceId,
        instrument_id: InstrumentId,
        pan: f32,
        attack_s: f32,
        decay_s: f32,
        sustain: f32,
        release_s: f32,
    ) -> Box<dyn VoiceTrait> {
        let synth_node = self.create_synth_node(instrument_id);
        let envelope = self.create_envelope(attack_s, decay_s, sustain, release_s);
        let voice = Voice::new(voice_id, synth_node, envelope, pan);

        Box::new(voice)
    }

    fn create_synth_node(&self, instrument_id: InstrumentId) -> impl SynthNode {
        // In a real implementation, this would look up the instrument definition.
        // For this example, we'll use a simple OscillatorNode.
        match instrument_id {
            // INSTRUMENT_ID_OSCILLATOR => Box::new(OscillatorNode::new(self.sample_rate)),
            // INSTRUMENT_ID_SUPERSAW => Box::new(SuperSawNode::new(self.sample_rate)),
            _ => OscillatorNode::new(), // Default
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
