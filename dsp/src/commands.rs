use crate::id::{EffectId, EnvelopeId, VoiceId};
use crate::instruments::Waveform;

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
