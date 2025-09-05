use std::vec;

use crate::{
    id::VoiceId, synth_infra::synth_node::SynthNode, Envelope, MonoEffect, MonoEffectChain,
    SynthCommand,
};

/// A trait for a generic, type-erased `Voice`. This is used for dynamic dispatch
/// in the `VoiceManager` to hold a heterogeneous collection of voices.
pub trait VoiceTrait: Send + Sync {
    /// Returns the unique ID of this voice instance.
    fn id(&self) -> VoiceId;

    /// Processes the next block of audio, adding the voice's output to the provided stereo buffers.
    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], frame_rate: f32);

    /// Triggers the note on event for this voice.
    fn note_on(&mut self, note: u8, velocity: u8);

    /// Triggers the note off event, usually starting the envelope's release phase.
    fn note_off(&mut self);

    /// Indicates if the voice as a whole is still active. A voice is considered
    /// inactive only after its envelope has completed its release phase and/or
    /// its underlying `SynthNode` has finished.
    fn is_active(&self) -> bool;

    /// Sets the stereo pan for this voice.
    fn set_pan(&mut self, pan: f32);

    /// Try to handle a synth-specific command
    fn try_handle_command(&mut self, command: &SynthCommand) -> bool;

    /// Add a mono effect to this voice's effect chain.
    fn add_effect(&mut self, effect: Box<dyn MonoEffect>);

    /// Set effect parameter
    fn set_effect_parameter(&mut self, effect_index: usize, param_index: u32, value: f32);
}

/// A `Voice` represents a single, monophonic musical event. It bundles a sound
/// generator (`SynthNode`) with its own dedicated `Envelope` and other state.
/// Polyphony is achieved by managing multiple `Voice` instances.
pub struct Voice<S: SynthNode> {
    id: VoiceId,
    pub(crate) node: S,
    envelope: Envelope,
    pan: f32, // -1.0 (L) to 1.0 (R)
    // Pre-allocated buffer for mono processing.
    mono_buf: Vec<f32>,
    /// Per voice effect chain.
    effect_chain: MonoEffectChain,
}

impl<S: SynthNode> Voice<S> {
    pub fn new(
        id: VoiceId,
        node: S,
        envelope: Envelope,
        // sample_rate: f32,
        pan: f32,
        effect_chain: MonoEffectChain,
    ) -> Self {
        // Pre-allocate the internal mono buffer for the voice.
        const MAX_BUFFER_SIZE: usize = 4096;
        let mono_buf = vec![0.0; MAX_BUFFER_SIZE];

        Self {
            id,
            node,
            envelope,
            pan,
            mono_buf,
            effect_chain,
        }
    }
}

// Implementation of the object-safe trait for the generic Voice.
impl<S: SynthNode> VoiceTrait for Voice<S> {
    fn id(&self) -> VoiceId {
        self.id
    }

    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
        let frame_count = left_buf.len();
        let mono_processing_buf = &mut self.mono_buf[..frame_count];

        // 1. Generate mono audio from the synth node.
        self.node.process(mono_processing_buf, sample_rate);

        // 2. Process the mono signal through the per-voice insert effects.
        //    Since our Effect trait works on stereo buffers, we pass the mono buffer
        //    for both left and right channels. The effect will process it in-place.
        self.effect_chain.process(mono_processing_buf, sample_rate);

        // 3. Calculate constant-power panning gains.
        let pan_angle = (self.pan + 1.0) * std::f32::consts::FRAC_PI_4; // Map [-1, 1] to [0, PI/2]
        let gain_left = pan_angle.cos();
        let gain_right = pan_angle.sin();

        // 4. Apply envelope and panning, adding to the main stereo buffers.
        for i in 0..frame_count {
            let envelope_val = self.envelope.process();
            let mono_sample = mono_processing_buf[i] * envelope_val;
            left_buf[i] += mono_sample * gain_left;
            right_buf[i] += mono_sample * gain_right;
        }
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        // Reset per-voice insert effects to avoid carrying state between notes
        // self.effect_chain.reset();
        self.node.note_on(note, velocity);
        self.envelope.gate(true);
    }

    fn note_off(&mut self) {
        self.node.note_off();
        self.envelope.gate(false);
    }

    fn is_active(&self) -> bool {
        self.envelope.is_active() && self.node.is_active()
    }

    fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    fn try_handle_command(&mut self, command: &SynthCommand) -> bool {
        self.node.try_handle_command(command)
    }

    fn add_effect(&mut self, effect: Box<dyn MonoEffect>) {
        self.effect_chain.add_effect(effect);
    }

    fn set_effect_parameter(&mut self, effect_index: usize, param_index: u32, value: f32) {
        self.effect_chain
            .set_effect_parameter(effect_index, param_index, value);
    }
}

/// Manages a heterogeneous collection of voices using dynamic dispatch.
#[deprecated(note = "This is needs to be presented behind a feature flag")]
#[allow(dead_code)]
pub struct VoiceManager {
    voices: Vec<Box<dyn VoiceTrait>>,
}

#[deprecated(note = "This is needs to be presented behind a feature flag")]
#[allow(dead_code, deprecated)]
impl VoiceManager {
    pub fn new() -> Self {
        Self {
            voices: Vec::with_capacity(64),
        }
    }

    pub fn find_voice_mut(&mut self, voice_id: VoiceId) -> Option<&mut Box<dyn VoiceTrait>> {
        self.voices.iter_mut().find(|v| v.id() == voice_id)
    }

    pub fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
        // Iterate and process active voices, which add their output to the buffers.
        for voice in self.voices.iter_mut() {
            voice.process(left_buf, right_buf, sample_rate); // Dynamic dispatch here.
        }

        // Remove inactive voices.
        self.voices.retain(|v| v.is_active());
    }

    pub fn add_voice(&mut self, voice: Box<dyn VoiceTrait>) {
        // In a real impl, this would use a more sophisticated voice allocation strategy.
        self.voices.push(voice);
    }
}
