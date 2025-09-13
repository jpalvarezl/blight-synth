use std::collections::HashMap;

use log::debug;
use sequencer::models::{MAX_TRACKS, NO_INSTRUMENT};

use crate::{
    id::InstrumentId, EngineCmd, InstrumentTrait, MixerCmd, MonoEffect, StereoEffectChain,
};

/// Specific implementation of a synthesizer for `tracker` mode.
/// Instruments are pre-allocated to prevent RT contract violations.
pub struct Synthesizer {
    pub instrument_bank: HashMap<InstrumentId, Box<dyn InstrumentTrait>>,
    pub track_last_instrument: HashMap<usize, InstrumentId>, // cache the active instrument for each track
    pub master_effects: StereoEffectChain,
    // TODO: future: consider per-instrument bus FX chains (StereoEffectChain per instrument)
    // and support a SequencerCmd::AddStereoEffectToInstrument to process post-sum per instrument.
}

impl Synthesizer {
    pub fn new() -> Self {
        Self {
            instrument_bank: HashMap::with_capacity(64),
            track_last_instrument: HashMap::with_capacity(MAX_TRACKS),
            master_effects: StereoEffectChain::new(8),
        }
    }

    pub fn note_on(&mut self, instrument_id: InstrumentId, note: u8, velocity: u8) {
        debug!("Playing note: {} on instrument: {}", note, instrument_id);
        debug!("Available instruments: {:?}", self.instrument_bank.keys());
        if let Some(instrument) = self.instrument_bank.get_mut(&instrument_id) {
            instrument.note_on(note, velocity);
        }
    }

    pub fn note_off(&mut self, instrument_id: InstrumentId) {
        if let Some(instrument) = self.instrument_bank.get_mut(&instrument_id) {
            instrument.note_off();
        }
    }

    pub fn process(&mut self, left: &mut [f32], right: &mut [f32], sample_rate: f32) {
        // 1. Process each instrument output
        for instrument in self.instrument_bank.values_mut() {
            instrument.process(left, right, sample_rate);
        }

        // 2. Process master effects
        self.master_effects.process(left, right, sample_rate);
    }

    pub fn add_instrument(&mut self, instrument: Box<dyn InstrumentTrait>) {
        self.instrument_bank.insert(instrument.id(), instrument);
    }

    pub fn add_effect_to_instrument(
        &mut self,
        instrument_id: InstrumentId,
        effect: Box<dyn MonoEffect>,
    ) {
        if let Some(instrument) = self.instrument_bank.get_mut(&instrument_id) {
            instrument.add_effect(effect);
        }
    }

    pub fn add_voice_effects_to_instrument(
        &mut self,
        instrument_id: InstrumentId,
        effects: crate::VoiceEffects,
    ) {
        if let Some(instrument) = self.instrument_bank.get_mut(&instrument_id) {
            instrument.add_voice_effects(effects);
        }
    }

    pub fn stop_all_notes(&mut self) {
        for instrument in self.instrument_bank.values_mut() {
            instrument.note_off();
        }
    }

    pub fn handle_engine_command(&mut self, cmd: EngineCmd) {
        match cmd {
            EngineCmd::NoteOn {
                instrument_id,
                note,
                velocity,
            } => {
                if let Some(inst) = self.instrument_bank.get_mut(&instrument_id) {
                    inst.note_on(note, velocity);
                }
            }
            EngineCmd::NoteOff { instrument_id } => {
                if let Some(inst) = self.instrument_bank.get_mut(&instrument_id) {
                    inst.note_off();
                }
            }
        }
    }

    pub fn handle_mixer_command(&mut self, cmd: MixerCmd) {
        match cmd {
            MixerCmd::AddMasterEffect { effect } => self.master_effects.add_effect(effect),
            MixerCmd::RemoveEffect { .. } => {}
            MixerCmd::ReorderEffects { .. } => {}
            MixerCmd::SetEffectParameter {
                instrument_id,
                effect_index,
                param_index,
                value,
            } => {
                if let Some(inst) = self.instrument_bank.get_mut(&instrument_id) {
                    inst.set_effect_parameter(effect_index, param_index, value);
                }
            }
        }
    }

    pub fn get_last_instrument_for_track(&mut self, track_index: &usize) -> InstrumentId {
        match self.track_last_instrument.get(track_index) {
            Some(&inst_id) => inst_id,
            None => NO_INSTRUMENT as InstrumentId, 
        }
    }

    pub fn set_last_instrument_for_track(&mut self, track_index: usize, instrument_id: InstrumentId) {
        self.track_last_instrument.insert(track_index, instrument_id);
    }
}
