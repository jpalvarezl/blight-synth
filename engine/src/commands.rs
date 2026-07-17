use dsp::{
    id::{EffectId, InstrumentId},
    InstrumentTrait, MonoEffect, StereoEffect, SynthCmd, VoiceEffects,
};

/// Commands that target one instrument and its owned voice/effect state.
#[allow(
    clippy::large_enum_variant,
    reason = "VoiceEffects stays inline to avoid container allocation and deallocation on the audio thread"
)]
pub enum InstrumentCmd {
    AddInstrument {
        instrument: Box<dyn InstrumentTrait>,
    },
    AddEffect {
        instrument_id: InstrumentId,
        effect: Box<dyn MonoEffect>,
    },
    AddVoiceEffects {
        instrument_id: InstrumentId,
        effects: VoiceEffects,
    },
    NoteOn {
        instrument_id: InstrumentId,
        note: u8,
        velocity: u8,
    },
    NoteOff {
        instrument_id: InstrumentId,
    },
    PassOnSynthCmd {
        instrument_id: InstrumentId,
        synth_cmd: SynthCmd,
    },
    SetEffectParameter {
        instrument_id: InstrumentId,
        effect_id: EffectId,
        param_index: u32,
        value: f32,
    },
}

/// Commands for the master mixer/effect pipeline.
///
/// Instrument-owned effects are deliberately excluded and belong to
/// [`InstrumentCmd`].
pub enum MixerCmd {
    AddMasterEffect {
        effect: Box<dyn StereoEffect>,
    },
    SetMasterEffectParameter {
        effect_id: EffectId,
        param_index: u32,
        value: f32,
    },
    RemoveMasterEffect {
        effect_index: usize,
    },
    ReorderMasterEffects {
        from_index: usize,
        to_index: usize,
    },
}

/// Transitional control-plane command grouping for the render engine.
///
/// This is not the final timestamped musical event API; M1 owns that contract.
#[allow(
    clippy::large_enum_variant,
    reason = "EngineCommand contains the intentionally inline InstrumentCmd payload"
)]
pub enum EngineCommand {
    Instrument(InstrumentCmd),
    Mixer(MixerCmd),
}

impl From<InstrumentCmd> for EngineCommand {
    fn from(value: InstrumentCmd) -> Self {
        Self::Instrument(value)
    }
}

impl From<MixerCmd> for EngineCommand {
    fn from(value: MixerCmd) -> Self {
        Self::Mixer(value)
    }
}
