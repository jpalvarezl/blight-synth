use crate::{
    id::{EffectChainId, InstrumentId, VoiceId},
    instruments::Waveform,
    InstrumentTrait, MonoEffect, StereoEffect, VoiceEffects, VoiceTrait,
};
use sequencer::models::Song;
use std::sync::Arc;

pub enum TransportCmd {
    PlayLastSong,
    StopSong,
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
    // Note/Voice Control
    PlayNoteInstrument {
        voice_id: VoiceId,
        note: u8,
        velocity: u8,
    },
    PlayNote {
        voice: Box<dyn VoiceTrait>,
        note: u8,
        velocity: u8,
    },
    StopNote {
        voice_id: VoiceId,
    },
    // Parameter Control (Type-Safe)
    SetVoicePan {
        voice_id: VoiceId,
        pan: f32,
    },
    SetSuperSawDetune {
        voice_id: VoiceId,
        detune: f32,
    },
    ChangeWaveform {
        voice_id: VoiceId,
        waveform: Waveform,
    },
    AddVoiceEffect {
        voice_id: VoiceId,
        effect: Box<dyn MonoEffect>,
    },
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

pub enum EngineCmd {
    NoteOn {
        instrument_id: InstrumentId,
        note: u8,
        velocity: u8,
    },
    NoteOff {
        instrument_id: InstrumentId,
    },
}

pub enum Command {
    Transport(TransportCmd),
    Sequencer(SequencerCmd),
    Synth(SynthCmd),
    Mixer(MixerCmd),
    Engine(EngineCmd),
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
impl From<SynthCmd> for Command {
    fn from(value: SynthCmd) -> Self {
        Command::Synth(value)
    }
}
impl From<MixerCmd> for Command {
    fn from(value: MixerCmd) -> Self {
        Command::Mixer(value)
    }
}
impl From<EngineCmd> for Command {
    fn from(value: EngineCmd) -> Self {
        Command::Engine(value)
    }
}
