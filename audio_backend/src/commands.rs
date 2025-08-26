use crate::{
    id::{EffectChainId, VoiceId},
    instruments::Waveform,
    MonoEffect, StereoEffect, VoiceTrait,
};
use crate::{id::InstrumentId, InstrumentTrait};
use std::sync::Arc;
use sequencer::models::Song;

pub enum Command {
    // Note/Voice Control
    PlayNoteInstrument {
        voice_id: VoiceId,
        note: u8,
        velocity: u8,
    },
    PlayNote {
        // voice_id: VoiceId, // Unique ID for this specific note event
        voice: Box<dyn VoiceTrait>,
        note: u8,
        velocity: u8,
        // instrument_id: InstrumentId, // Specifies which synth/sample to use
        // pan: f32,
    },
    StopNote {
        voice_id: VoiceId, // Target a specific note to stop
    },
    // Parameter Control (Type-Safe)
    SetVoicePan {
        voice_id: VoiceId,
        pan: f32,
    },
    // Example of a type-specific parameter command
    SetSuperSawDetune {
        voice_id: VoiceId,
        detune: f32,
    },
    // SetParameter { param: String, value: f32 },
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
    ChangeWaveform {
        voice_id: VoiceId,
        waveform: Waveform,
    },
    AddMasterEffect {
        effect: Box<dyn StereoEffect>, // The Box is created in the NRT world
    },
    AddVoiceEffect {
        voice_id: VoiceId,
        effect: Box<dyn MonoEffect>, // The Box is created in the NRT world
    },
    // Tracker/Sequencer control
    AddTrackInstrument {
        instrument: Box<dyn InstrumentTrait>,
    },
    AddEffectToInstrument {
        instrument_id: InstrumentId,
        effect: Box<dyn StereoEffect>,
    },
    PlaySong {
        song: Arc<Song>,
    },
    PlayLastSong,
    StopSong,
    // Add more commands as needed.
}
