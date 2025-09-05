use crate::id::InstrumentId;
use crate::{instruments::VoiceSlot, OscillatorNode};
use crate::{
    Envelope, InstrumentTrait, MonoEffect, MonoEffectChain, Voice, VoiceEffects, VoiceTrait,
};

pub struct PolyphonicOscillator {
    instrument_id: InstrumentId,
    voices: Vec<VoiceSlot<OscillatorNode>>,
}

impl PolyphonicOscillator {
    pub fn new(instrument_id: InstrumentId, pan: f32, sample_rate: f32, max_polyphony: u8) -> Self {
        let mut envelope = Envelope::new(sample_rate);
        envelope.set_parameters(0.1, 0.1, 0.8, 0.1); // Default ADSR values
        let voices: Vec<VoiceSlot<OscillatorNode>> = (0..max_polyphony)
            .map(|_| VoiceSlot {
                note_id: None,
                inner: Voice::new(
                    0,
                    OscillatorNode::new(),
                    envelope.clone(),
                    pan,
                    MonoEffectChain::new(10),
                ),
            })
            .collect();

        log::info!("Allocating PolyphonicOscillator with ID: {}", instrument_id);
        for voice in &voices {
            log::info!(
                "Created voice with ID: {}, state: {}",
                voice.inner.id(),
                voice.inner.is_active()
            );
        }
        log::info!("Allocation complete");

        PolyphonicOscillator {
            instrument_id,
            voices,
        }
    }
}

impl InstrumentTrait for PolyphonicOscillator {
    fn id(&self) -> crate::id::InstrumentId {
        self.instrument_id
    }

    fn note_on(&mut self, note: u8, velocity: u8) {
        log::info!("Looking for voice");
        for voice in &self.voices {
            log::info!(
                "Checking voice with ID: {:#?}, state: {}",
                voice.note_id,
                voice.inner.is_active()
            );
        }
        // Find a free voice or a voice with the same note to retrigger envelope
        let free_voice = self
            .voices
            .iter_mut()
            .find(|slot| !slot.inner.is_active() || slot.note_id == Some(note));

        log::info!(
            "Free voice found: {:?}, ID is: {:?}",
            free_voice.is_some(),
            free_voice.as_ref().map(|slot| slot.note_id)
        );

        if let Some(slot) = free_voice {
            // We found a free voice, so we can use it.
            slot.note_id = Some(note);
            slot.inner.note_on(note, velocity);
        } else {
            // No free voices. This is where you would implement voice stealing.
            // For now, we'll just ignore the new note.
        }
    }

    /// This mutes all voices.
    fn note_off(&mut self) {
        for voice in &mut self.voices {
            if voice.inner.is_active() {
                voice.inner.note_off();
            }
        }
    }

    fn process(&mut self, left_buf: &mut [f32], right_buf: &mut [f32], sample_rate: f32) {
        // process active voices
        for voice in self.voices.iter_mut() {
            voice.inner.process(left_buf, right_buf, sample_rate);
        }
    }

    fn set_pan(&mut self, pan: f32) {
        for voice in &mut self.voices {
            voice.inner.set_pan(pan);
        }
    }

    fn add_effect(&mut self, _effect: Box<dyn MonoEffect>) {
        // Polyphonic instruments require one effect instance per voice.
        // Use add_voice_effects with pre-constructed per-voice effects instead.
        log::warn!("PolyphonicOscillator: add_effect is a no-op; use add_voice_effects instead");
    }

    fn add_voice_effects(&mut self, effects: VoiceEffects) {
        for (slot, effect) in self.voices.iter_mut().zip(effects.into_iter()) {
            slot.inner.add_effect(effect);
        }
    }

    fn set_effect_parameter(&mut self, effect_index: usize, param_index: u32, value: f32) {
        for voice in &mut self.voices {
            voice
                .inner
                .set_effect_parameter(effect_index, param_index, value);
        }
    }
}
