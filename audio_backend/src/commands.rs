use crate::{
    id::{EffectChainId, InstrumentId, VoiceId},
    instruments::Waveform,
    InstrumentTrait, MonoEffect, StereoEffect, VoiceTrait,
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
    AddEffectToInstrument {
        instrument_id: InstrumentId,
        effect: Box<dyn StereoEffect>,
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
        target_chain: EffectChainId,
        effect_index: usize,
        param_index: u32,
        value: f32,
    },
}

pub enum Command {
    Transport(TransportCmd),
    Sequencer(SequencerCmd),
    Synth(SynthCmd),
    Mixer(MixerCmd),
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
