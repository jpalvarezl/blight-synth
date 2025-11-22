use crate::{
    id::{EffectChainId, EffectId, EnvelopeId, InstrumentId, VoiceId},
    instruments::Waveform,
    InstrumentTrait, MonoEffect, StereoEffect, VoiceEffects,
};
use sequencer::models::Song;
use std::sync::Arc;

pub enum TransportCmd {
    PlayLastSong,
    StopSong,
    SetLooping { enabled: bool },
}

pub enum SequencerCmd {
    PlaySong {
        song: Arc<Song>,
    },
    AddTrackInstrument {
        instrument: Box<dyn InstrumentTrait>,
    },
    // TODO: consider a future AddStereoEffectToInstrument { instrument_id, effect: Box<dyn StereoEffect> }
    // for per-instrument bus FX processed after summing all voices.
    AddEffectToInstrument {
        instrument_id: InstrumentId,
        effect: Box<dyn MonoEffect>,
    },
    /// Install a batch of per-voice effects into an instrument in one RT-safe operation.
    AddVoiceEffectsToInstrument {
        instrument_id: InstrumentId,
        effects: VoiceEffects,
    },
}

pub enum SynthCmd {
    SetWaveform {
        voice_id: VoiceId,
        waveform: Waveform,
    },
    EnvelopeCommand {
        envelope_id: Option<EnvelopeId>,
        command: EnvelopeCmd,
    },
    EffectCommand {
        effect_id: EffectId,
        command: EffectCmd,
    },
}

pub enum EffectCmd {
    SetParameter { param_index: u32, value: f32 },
    SwapEffectOrder { target_effect_id: EffectId },
}

pub enum EnvelopeCmd {
    SetPitchEnvFreqDelta { freq_delta: f32 },
    SetAttack { attack: f32 },
    SetDecay { decay: f32 },
    SetSustain { sustain: f32 },
    SetRelease { release: f32 },
}

pub enum MixerCmd {
    AddMasterEffect {
        effect: Box<dyn StereoEffect>,
    },
    RemoveEffect {
        target_chain: EffectChainId,
        effect_index: usize,
    },
    ReorderEffects {
        target_chain: EffectChainId,
        from_index: usize,
        to_index: usize,
    },
    SetEffectParameter {
        instrument_id: InstrumentId,
        effect_index: usize,
        param_index: u32,
        value: f32,
    },
}

pub enum InstrumentCmd {
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
}

pub enum Command {
    Transport(TransportCmd),
    Sequencer(SequencerCmd),
    Mixer(MixerCmd),
    Instrument(InstrumentCmd),
}

impl From<TransportCmd> for Command {
    fn from(value: TransportCmd) -> Self {
        Command::Transport(value)
    }
}
impl From<SequencerCmd> for Command {
    fn from(value: SequencerCmd) -> Self {
        Command::Sequencer(value)
    }
}
impl From<MixerCmd> for Command {
    fn from(value: MixerCmd) -> Self {
        Command::Mixer(value)
    }
}
impl From<InstrumentCmd> for Command {
    fn from(value: InstrumentCmd) -> Self {
        Command::Instrument(value)
    }
}
