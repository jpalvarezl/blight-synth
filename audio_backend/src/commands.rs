use crate::{id::EffectChainId, id::VoiceId, synths::Waveform, VoiceTrait};

pub enum Command {
    // Note/Voice Control
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
    // AddEffect {
    //     target_chain: EffectChainId,
    //     effect: Box<dyn Effect>, // The Box is created in the NRT world
    // },
    // RemoveEffect {
    //     target_chain: EffectChainId,
    //     effect_index: usize,
    // },
    // ReorderEffects {
    //     target_chain: EffectChainId,
    //     from_index: usize,
    //     to_index: usize,
    // },
    // SetEffectParameter {
    //     target_chain: EffectChainId,
    //     effect_index: usize,
    //     param_index: u32,
    //     value: f32,
    // },
    // Add more commands as needed.
}
