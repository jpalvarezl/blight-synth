use audio_backend::EnvelopeCmd;
use eframe::egui;
use sequencer::models::{
    AmpEnvelopeParams, AudioEffect, HiHatParams, Instrument, InstrumentData, KickDrumParams,
    SimpleOscillatorParams, SnareDrumParams, Song, Waveform,
};

use crate::audio::AudioManager;
use crate::audio_utils::map_waveform_to_backend;

#[derive(Default)]
pub struct InstrumentManagerWindow {
    pub open: bool,
}

fn send_amp_envelope_to_backend(
    audio_mgr: &mut AudioManager,
    instrument_id: u8,
    env: &AmpEnvelopeParams,
) {
    if let Some(audio) = &mut audio_mgr.audio {
        let id = audio_backend::id::InstrumentId::from(instrument_id as u32);
        audio.send_command(
            audio_backend::InstrumentCmd::PassOnSynthCmd {
                instrument_id: id,
                synth_cmd: audio_backend::SynthCmd::EnvelopeCommand {
                    envelope_id: Some(0),
                    command: EnvelopeCmd::SetAttack { attack: env.attack },
                },
            }
            .into(),
        );
        audio.send_command(
            audio_backend::InstrumentCmd::PassOnSynthCmd {
                instrument_id: id,
                synth_cmd: audio_backend::SynthCmd::EnvelopeCommand {
                    envelope_id: Some(0),
                    command: EnvelopeCmd::SetDecay { decay: env.decay },
                },
            }
            .into(),
        );
        audio.send_command(
            audio_backend::InstrumentCmd::PassOnSynthCmd {
                instrument_id: id,
                synth_cmd: audio_backend::SynthCmd::EnvelopeCommand {
                    envelope_id: Some(0),
                    command: EnvelopeCmd::SetSustain {
                        sustain: env.sustain,
                    },
                },
            }
            .into(),
        );
        audio.send_command(
            audio_backend::InstrumentCmd::PassOnSynthCmd {
                instrument_id: id,
                synth_cmd: audio_backend::SynthCmd::EnvelopeCommand {
                    envelope_id: Some(0),
                    command: EnvelopeCmd::SetRelease {
                        release: env.release,
                    },
                },
            }
            .into(),
        );
    }
}

fn show_amp_envelope_controls(
    ui: &mut egui::Ui,
    params: &mut AmpEnvelopeParams,
    instrument_id: usize,
    ui_prefix: &'static str,
    audio_mgr: &mut AudioManager,
) {
    ui.push_id((ui_prefix, instrument_id as u32, "amp_env"), |ui| {
        egui::CollapsingHeader::new("Amplitude Envelope")
            .id_salt((ui_prefix, instrument_id as u32, "amp_env_hdr"))
            .show(ui, |ui| {
                let mut changed = false;
                let mut atk = params.attack;
                let mut dec = params.decay;
                let mut sus = params.sustain;
                let mut rel = params.release;

                ui.horizontal(|ui| {
                    ui.label("Attack");
                    changed |= ui
                        .add(egui::Slider::new(&mut atk, 0.0..=2.0).suffix(" s"))
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Decay");
                    changed |= ui
                        .add(egui::Slider::new(&mut dec, 0.0..=2.0).suffix(" s"))
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Sustain");
                    changed |= ui.add(egui::Slider::new(&mut sus, 0.0..=1.0)).changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Release");
                    changed |= ui
                        .add(egui::Slider::new(&mut rel, 0.0..=5.0).suffix(" s"))
                        .changed();
                });

                if changed {
                    params.attack = atk;
                    params.decay = dec;
                    params.sustain = sus;
                    params.release = rel;
                    send_amp_envelope_to_backend(audio_mgr, instrument_id as u8, params);
                }
            });
    });
}

/// Create/replace the backend oscillator instrument and configure its voice effects from params.
fn ensure_backend_osc_with_params(
    audio_mgr: &mut AudioManager,
    id_u8: u8,
    params: &SimpleOscillatorParams,
) {
    if let Some(audio) = &mut audio_mgr.audio {
        let backend_wave = map_waveform_to_backend(params.waveform);
        let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
        let instrument = audio
            .get_instrument_factory()
            .create_oscillator_with_waveform(id, 0.0, backend_wave);
        // Replace instrument in the backend
        audio.send_command(audio_backend::SequencerCmd::AddTrackInstrument { instrument }.into());

        // Configure mono insert effects from params
        for eff in &params.audio_effects {
            match eff {
                AudioEffect::Reverb {
                    mix,
                    decay_time,
                    room_size,
                    diffusion,
                    damping,
                } => {
                    let mut r = audio.get_effect_factory().create_mono_reverb();
                    // Reverb parameter enums for clarity and safety
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Mix.as_index(),
                        (*mix).clamp(0.0, 1.0),
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Decay.as_index(),
                        *decay_time,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::RoomSize.as_index(),
                        *room_size,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Damping.as_index(),
                        *damping,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Diffusion.as_index(),
                        *diffusion,
                    );

                    audio.send_command(
                        audio_backend::SequencerCmd::AddEffectToInstrument {
                            instrument_id: id,
                            effect: r,
                        }
                        .into(),
                    );
                }
                AudioEffect::Delay {
                    time,
                    num_taps,
                    feedback,
                    mix,
                } => {
                    // Create a mono delay with the configured taps and mix
                    let mut d = audio.get_effect_factory().create_mono_delay(
                        *time,
                        *num_taps as usize,
                        *feedback,
                        *mix,
                    );
                    // Explicitly set parameters using enum indices
                    use audio_backend::effects::DelayParameter as DP;
                    audio_backend::MonoEffect::set_parameter(&mut *d, DP::Time.as_index(), *time);
                    audio_backend::MonoEffect::set_parameter(
                        &mut *d,
                        DP::NumTaps.as_index(),
                        *num_taps as f32,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *d,
                        DP::Feedback.as_index(),
                        *feedback,
                    );
                    audio_backend::MonoEffect::set_parameter(&mut *d, DP::Mix.as_index(), *mix);

                    audio.send_command(
                        audio_backend::SequencerCmd::AddEffectToInstrument {
                            instrument_id: id,
                            effect: d,
                        }
                        .into(),
                    );
                }
            }
        }
    }

    send_amp_envelope_to_backend(audio_mgr, id_u8, &params.amp_envelope);
}

/// Create/replace the backend hi-hat instrument and configure its voice effects from params.
fn ensure_backend_hihat_with_params(audio_mgr: &mut AudioManager, id_u8: u8, params: &HiHatParams) {
    if let Some(audio) = &mut audio_mgr.audio {
        let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
        let instrument = audio.get_instrument_factory().create_hihat(id, 0.0);
        // Replace instrument in the backend
        audio.send_command(audio_backend::SequencerCmd::AddTrackInstrument { instrument }.into());

        // Configure mono insert effects from params
        for eff in &params.audio_effects {
            match eff {
                AudioEffect::Reverb {
                    mix,
                    decay_time,
                    room_size,
                    diffusion,
                    damping,
                } => {
                    let mut r = audio.get_effect_factory().create_mono_reverb();
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Mix.as_index(),
                        (*mix).clamp(0.0, 1.0),
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Decay.as_index(),
                        *decay_time,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::RoomSize.as_index(),
                        *room_size,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Damping.as_index(),
                        *damping,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Diffusion.as_index(),
                        *diffusion,
                    );

                    audio.send_command(
                        audio_backend::SequencerCmd::AddEffectToInstrument {
                            instrument_id: id,
                            effect: r,
                        }
                        .into(),
                    );
                }
                AudioEffect::Delay {
                    time,
                    num_taps,
                    feedback,
                    mix,
                } => {
                    let mut d = audio.get_effect_factory().create_mono_delay(
                        *time,
                        *num_taps as usize,
                        *feedback,
                        *mix,
                    );
                    use audio_backend::effects::DelayParameter as DP;
                    audio_backend::MonoEffect::set_parameter(&mut *d, DP::Time.as_index(), *time);
                    audio_backend::MonoEffect::set_parameter(
                        &mut *d,
                        DP::NumTaps.as_index(),
                        *num_taps as f32,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *d,
                        DP::Feedback.as_index(),
                        *feedback,
                    );
                    audio_backend::MonoEffect::set_parameter(&mut *d, DP::Mix.as_index(), *mix);

                    audio.send_command(
                        audio_backend::SequencerCmd::AddEffectToInstrument {
                            instrument_id: id,
                            effect: d,
                        }
                        .into(),
                    );
                }
            }
        }
    }

    send_amp_envelope_to_backend(audio_mgr, id_u8, &params.amp_envelope);
}

/// Create/replace the backend kick drum instrument and configure its voice effects from params.
fn ensure_backend_kick_with_params(
    audio_mgr: &mut AudioManager,
    id_u8: u8,
    params: &KickDrumParams,
) {
    if let Some(audio) = &mut audio_mgr.audio {
        let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
        let instrument = audio.get_instrument_factory().create_kick_drum(id, 0.0);
        // Replace instrument in the backend
        audio.send_command(audio_backend::SequencerCmd::AddTrackInstrument { instrument }.into());

        // Configure mono insert effects from params
        for eff in &params.audio_effects {
            match eff {
                AudioEffect::Reverb {
                    mix,
                    decay_time,
                    room_size,
                    diffusion,
                    damping,
                } => {
                    let mut r = audio.get_effect_factory().create_mono_reverb();
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Mix.as_index(),
                        (*mix).clamp(0.0, 1.0),
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Decay.as_index(),
                        *decay_time,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::RoomSize.as_index(),
                        *room_size,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Damping.as_index(),
                        *damping,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Diffusion.as_index(),
                        *diffusion,
                    );
                    audio.send_command(
                        audio_backend::SequencerCmd::AddEffectToInstrument {
                            instrument_id: id,
                            effect: r,
                        }
                        .into(),
                    );
                }
                AudioEffect::Delay {
                    time,
                    num_taps,
                    feedback,
                    mix,
                } => {
                    let mut d = audio.get_effect_factory().create_mono_delay(
                        *time,
                        *num_taps as usize,
                        *feedback,
                        *mix,
                    );
                    use audio_backend::effects::DelayParameter as DP;
                    audio_backend::MonoEffect::set_parameter(&mut *d, DP::Time.as_index(), *time);
                    audio_backend::MonoEffect::set_parameter(
                        &mut *d,
                        DP::NumTaps.as_index(),
                        *num_taps as f32,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *d,
                        DP::Feedback.as_index(),
                        *feedback,
                    );
                    audio_backend::MonoEffect::set_parameter(&mut *d, DP::Mix.as_index(), *mix);
                    audio.send_command(
                        audio_backend::SequencerCmd::AddEffectToInstrument {
                            instrument_id: id,
                            effect: d,
                        }
                        .into(),
                    );
                }
            }
        }
        // Configure pitch envelope parameters (simplified for typical kick pitch sweep)
        audio.send_command(
            audio_backend::InstrumentCmd::PassOnSynthCmd {
                instrument_id: id,
                synth_cmd: audio_backend::SynthCmd::EnvelopeCommand {
                    envelope_id: None,
                    command: EnvelopeCmd::SetPitchEnvFreqDelta {
                        freq_delta: params.pitch_envelope.freq_delta,
                    },
                },
            }
            .into(),
        );
    }

    send_amp_envelope_to_backend(audio_mgr, id_u8, &params.amp_envelope);
}

/// Create/replace the backend snare drum instrument and configure its voice effects from params.
fn ensure_backend_snare_with_params(
    audio_mgr: &mut AudioManager,
    id_u8: u8,
    params: &SnareDrumParams,
) {
    if let Some(audio) = &mut audio_mgr.audio {
        let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
        let instrument = audio.get_instrument_factory().create_snare_drum(id, 0.0);
        // Replace instrument in the backend
        audio.send_command(audio_backend::SequencerCmd::AddTrackInstrument { instrument }.into());

        // Configure mono insert effects from params
        for eff in &params.audio_effects {
            match eff {
                AudioEffect::Reverb {
                    mix,
                    decay_time,
                    room_size,
                    diffusion,
                    damping,
                } => {
                    let mut r = audio.get_effect_factory().create_mono_reverb();
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Mix.as_index(),
                        (*mix).clamp(0.0, 1.0),
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Decay.as_index(),
                        *decay_time,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::RoomSize.as_index(),
                        *room_size,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Damping.as_index(),
                        *damping,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Diffusion.as_index(),
                        *diffusion,
                    );
                    audio.send_command(
                        audio_backend::SequencerCmd::AddEffectToInstrument {
                            instrument_id: id,
                            effect: r,
                        }
                        .into(),
                    );
                }
                AudioEffect::Delay {
                    time,
                    num_taps,
                    feedback,
                    mix,
                } => {
                    let mut d = audio.get_effect_factory().create_mono_delay(
                        *time,
                        *num_taps as usize,
                        *feedback,
                        *mix,
                    );
                    use audio_backend::effects::DelayParameter as DP;
                    audio_backend::MonoEffect::set_parameter(&mut *d, DP::Time.as_index(), *time);
                    audio_backend::MonoEffect::set_parameter(
                        &mut *d,
                        DP::NumTaps.as_index(),
                        *num_taps as f32,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *d,
                        DP::Feedback.as_index(),
                        *feedback,
                    );
                    audio_backend::MonoEffect::set_parameter(&mut *d, DP::Mix.as_index(), *mix);
                    audio.send_command(
                        audio_backend::SequencerCmd::AddEffectToInstrument {
                            instrument_id: id,
                            effect: d,
                        }
                        .into(),
                    );
                }
            }
        }
    }

    send_amp_envelope_to_backend(audio_mgr, id_u8, &params.amp_envelope);
}

fn waveform_display_name(w: Waveform) -> &'static str {
    match w {
        Waveform::Sine => "Sine",
        Waveform::Square => "Square",
        Waveform::Sawtooth => "Sawtooth",
        Waveform::Triangle => "Triangle",
        Waveform::NesTriangle => "NES Triangle",
    }
}

impl InstrumentManagerWindow {
    fn next_free_instrument_id(song: &Song) -> u8 {
        for id in 1u16..=255u16 {
            if !song.instrument_bank.iter().any(|i| i.id == id as usize) {
                return id as u8;
            }
        }
        1
    }

    pub fn show(&mut self, ctx: &egui::Context, song: &mut Song, audio_mgr: &mut AudioManager) {
        if !self.open {
            return;
        }
        let mut to_add_osc = false;
        let mut to_add_hihat = false;
        let mut to_add_kick = false;
        let mut to_add_snare = false;
        let mut to_add_dfam = false;
        // Track instruments needing backend rehydration this frame
        let mut rehydrate_ids: Vec<u8> = Vec::new();
        egui::Window::new("Instruments")
            .open(&mut self.open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Add Oscillator").clicked() {
                        to_add_osc = true;
                    }
                    if ui.button("Add Hi-Hat").clicked() {
                        to_add_hihat = true;
                    }
                    if ui.button("Add Kick Drum").clicked() {
                        to_add_kick = true;
                    }
                    if ui.button("Add Snare Drum").clicked() {
                        to_add_snare = true;
                    }
                    if ui.button("Add DFAM").clicked() {
                        to_add_dfam = true;
                    }
                });

                ui.separator();

                if song.instrument_bank.is_empty() {
                    ui.label("No instruments. Click 'Add Oscillator' to create one.");
                } else {
                    for inst in song.instrument_bank.iter_mut() {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("ID {:02X}", inst.id as u8));
                                ui.text_edit_singleline(&mut inst.name);
                            });
                            match &mut inst.data {
                                InstrumentData::SimpleOscillator(params) => {
                                    ui.horizontal(|ui| {
                                        ui.label("Waveform:");
                                        let mut wf = params.waveform;
                                        egui::ComboBox::from_id_salt(("wf", inst.id))
                                            .selected_text(waveform_display_name(wf))
                                            .show_ui(ui, |ui| {
                                                for w in [
                                                    Waveform::Sine,
                                                    Waveform::Square,
                                                    Waveform::Sawtooth,
                                                    Waveform::Triangle,
                                                    Waveform::NesTriangle,
                                                ] {
                                                    if ui
                                                        .selectable_label(
                                                            wf == w,
                                                            waveform_display_name(w),
                                                        )
                                                        .clicked()
                                                    {
                                                        wf = w;
                                                    }
                                                }
                                            });
                                        if wf != params.waveform {
                                            params.waveform = wf;
                                            rehydrate_ids.push(inst.id as u8);
                                        }
                                    });

                                            ui.separator();
                                            show_amp_envelope_controls(
                                                ui,
                                                &mut params.amp_envelope,
                                                inst.id,
                                                "osc",
                                                audio_mgr,
                                            );
                                    ui.separator();
                                    ui.label("Effects:");

                                    // Reverb controls (single instance per mono instrument)
                                    let mut has_reverb = false;
                                    let mut to_remove_reverb = false;
                                    for (eff_idx, eff) in params.audio_effects.iter_mut().enumerate() {
                                        if let AudioEffect::Reverb {
                                            mix,
                                            decay_time,
                                            room_size,
                                            diffusion,
                                            damping,
                                        } = eff
                                        {
                                            has_reverb = true;
                                            // Namespace all inner widgets by instrument id to avoid ID clashes
                                            ui.push_id(("reverb", inst.id as u32), |ui| {
                                                egui::CollapsingHeader::new(format!(
                                                    "Reverb {:02X}",
                                                    inst.id as u8
                                                ))
                                                .id_salt(("reverb_hdr", inst.id as u32))
                                                .show(ui, |ui| {
                                                    let mut changed = false;
                                                    let mut mx = *mix;
                                                    let mut dec = *decay_time;
                                                    let mut rs = *room_size;
                                                    let mut diff = *diffusion;
                                                    let mut damp = *damping;
                                                    ui.horizontal(|ui| {
                                                        ui.label("Mix");
                                                        changed |= ui
                                                            .add(egui::Slider::new(
                                                                &mut mx,
                                                                0.0..=1.0,
                                                            ))
                                                            .changed();
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Decay");
                                                        changed |= ui
                                                            .add(egui::Slider::new(
                                                                &mut dec,
                                                                0.0..=1.0,
                                                            ))
                                                            .changed();
                                                        ui.label("Damping");
                                                        changed |= ui
                                                            .add(egui::Slider::new(
                                                                &mut damp,
                                                                0.0..=1.0,
                                                            ))
                                                            .changed();
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Room Size");
                                                        changed |= ui
                                                            .add(egui::Slider::new(
                                                                &mut rs,
                                                                0.5..=2.0,
                                                            ))
                                                            .changed();
                                                        ui.label("Diffusion");
                                                        changed |= ui
                                                            .add(egui::Slider::new(
                                                                &mut diff,
                                                                0.0..=1.0,
                                                            ))
                                                            .changed();
                                                    });
                                                    if changed {
                                                        // 1) Update the song model
                                                        *mix = mx;
                                                        *decay_time = dec;
                                                        *room_size = rs;
                                                        *diffusion = diff;
                                                        *damping = damp;

                                                        // 2) Live-update backend via MixerCmd
                                                        if let Some(audio) = &mut audio_mgr.audio {
                                                            let id = audio_backend::id::InstrumentId::from(inst.id as u32);
                                                            use audio_backend::effects::ReverbParameter as RP;
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: RP::Mix.as_index(),
                                                                    value: mx,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: RP::Decay.as_index(),
                                                                    value: dec,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: RP::RoomSize.as_index(),
                                                                    value: rs,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: RP::Damping.as_index(),
                                                                    value: damp,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: RP::Diffusion.as_index(),
                                                                    value: diff,
                                                                }
                                                                .into(),
                                                            );
                                                        }
                                                    }
                                                    if ui.button("Remove Reverb").clicked() {
                                                        to_remove_reverb = true;
                                                    }
                                                });
                                            });
                                        }
                                    }
                                    if to_remove_reverb {
                                        params
                                            .audio_effects
                                            .retain(|e| !matches!(e, AudioEffect::Reverb { .. }));
                                        rehydrate_ids.push(inst.id as u8);
                                    }

                                    if !has_reverb {
                                        // Namespace the Add button too
                                        ui.push_id(("add_reverb", inst.id as u32), |ui| {
                                            if ui.button("Add Reverb").clicked() {
                                                params.audio_effects.push(AudioEffect::Reverb {
                                                    mix: 0.3,
                                                    decay_time: 0.6,
                                                    room_size: 1.0,
                                                    diffusion: 1.0,
                                                    damping: 0.2,
                                                });
                                                rehydrate_ids.push(inst.id as u8);
                                            }
                                        });
                                    }

                                    // Delay controls (single instance per mono instrument)
                                    let mut has_delay = false;
                                    let mut to_remove_delay = false;
                                    for (eff_idx, eff) in params.audio_effects.iter_mut().enumerate() {
                                        if let AudioEffect::Delay {
                                            time,
                                            num_taps,
                                            feedback,
                                            mix,
                                        } = eff
                                        {
                                            has_delay = true;
                                            // Namespace all inner widgets by instrument id to avoid ID clashes
                                            ui.push_id(("delay", inst.id as u32), |ui| {
                                                egui::CollapsingHeader::new(format!(
                                                    "Delay {:02X}",
                                                    inst.id as u8
                                                ))
                                                .id_salt(("delay_hdr", inst.id as u32))
                                                .show(ui, |ui| {
                                                    let mut changed = false;
                                                    let mut t = *time;
                                                    let mut tp = *num_taps;
                                                    let mut fb = *feedback;
                                                    let mut mx = *mix;
                                                    ui.horizontal(|ui| {
                                                        ui.label("Time (s)");
                                                        changed |= ui
                                                            .add(egui::Slider::new(
                                                                &mut t,
                                                                0.0..=audio_backend::effects::MAX_DELAY_SECONDS,
                                                            ))
                                                            .changed();
                                                        ui.label("Feedback");
                                                        changed |= ui
                                                            .add(egui::Slider::new(
                                                                &mut fb,
                                                                0.0..=0.95,
                                                            ))
                                                            .changed();
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Taps");
                                                        changed |= ui
                                                            .add(egui::Slider::new(
                                                                &mut tp,
                                                                1..=audio_backend::effects::MAX_TAPS as u8,
                                                            ))
                                                            .changed();
                                                        ui.label("Mix");
                                                        changed |= ui
                                                            .add(egui::Slider::new(
                                                                &mut mx,
                                                                0.0..=1.0,
                                                            ))
                                                            .changed();
                                                    });
                                                    if changed {
                                                        // 1) Update in the Song model
                                                        *time = t;
                                                        *num_taps = tp;
                                                        *feedback = fb;
                                                        *mix = mx;

                                                        // 2) Live-update backend via MixerCmd
                                                        if let Some(audio) = &mut audio_mgr.audio {
                                                            let id = audio_backend::id::InstrumentId::from(inst.id as u32);
                                                            use audio_backend::effects::DelayParameter as DP;
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: DP::Time.as_index(),
                                                                    value: t,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: DP::NumTaps.as_index(),
                                                                    value: tp as f32,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: DP::Feedback.as_index(),
                                                                    value: fb,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: DP::Mix.as_index(),
                                                                    value: mx,
                                                                }
                                                                .into(),
                                                            );
                                                        }
                                                    }
                                                    if ui.button("Remove Delay").clicked() {
                                                        to_remove_delay = true;
                                                    }
                                                });
                                            });
                                        }
                                    }
                                    if to_remove_delay {
                                        params
                                            .audio_effects
                                            .retain(|e| !matches!(e, AudioEffect::Delay { .. }));
                                        rehydrate_ids.push(inst.id as u8);
                                    }

                                    if !has_delay {
                                        // Namespace the Add button too
                                        ui.push_id(("add_delay", inst.id as u32), |ui| {
                                            if ui.button("Add Delay").clicked() {
                                                params.audio_effects.push(AudioEffect::Delay {
                                                    time: 0.3,
                                                    num_taps: 3,
                                                    feedback: 0.3,
                                                    mix: 0.35,
                                                });
                                                rehydrate_ids.push(inst.id as u8);
                                            }
                                        });
                                    }
                                }
                                InstrumentData::HiHat(params) => {
                                    ui.label("Hi-Hat");
                                    ui.separator();
                                    show_amp_envelope_controls(
                                        ui,
                                        &mut params.amp_envelope,
                                        inst.id,
                                        "hh",
                                        audio_mgr,
                                    );
                                    ui.separator();
                                    ui.label("Effects:");

                                    // Reverb controls
                                    let mut has_reverb = false;
                                    let mut to_remove_reverb = false;
                                    for (eff_idx, eff) in params.audio_effects.iter_mut().enumerate() {
                                        if let AudioEffect::Reverb {
                                            mix,
                                            decay_time,
                                            room_size,
                                            diffusion,
                                            damping,
                                        } = eff
                                        {
                                            has_reverb = true;
                                            ui.push_id(("hh_reverb", inst.id as u32), |ui| {
                                                egui::CollapsingHeader::new(format!(
                                                    "Reverb {:02X}",
                                                    inst.id as u8
                                                ))
                                                .id_salt(("hh_reverb_hdr", inst.id as u32))
                                                .show(ui, |ui| {
                                                    let mut changed = false;
                                                    let mut mx = *mix;
                                                    let mut dec = *decay_time;
                                                    let mut rs = *room_size;
                                                    let mut diff = *diffusion;
                                                    let mut damp = *damping;
                                                    ui.horizontal(|ui| {
                                                        ui.label("Mix");
                                                        changed |= ui
                                                            .add(egui::Slider::new(&mut mx, 0.0..=1.0))
                                                            .changed();
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Decay");
                                                        changed |= ui
                                                            .add(egui::Slider::new(&mut dec, 0.0..=1.0))
                                                            .changed();
                                                        ui.label("Damping");
                                                        changed |= ui
                                                            .add(egui::Slider::new(&mut damp, 0.0..=1.0))
                                                            .changed();
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Room Size");
                                                        changed |= ui
                                                            .add(egui::Slider::new(&mut rs, 0.5..=2.0))
                                                            .changed();
                                                        ui.label("Diffusion");
                                                        changed |= ui
                                                            .add(egui::Slider::new(&mut diff, 0.0..=1.0))
                                                            .changed();
                                                    });
                                                    if changed {
                                                        *mix = mx;
                                                        *decay_time = dec;
                                                        *room_size = rs;
                                                        *diffusion = diff;
                                                        *damping = damp;

                                                        if let Some(audio) = &mut audio_mgr.audio {
                                                            let id = audio_backend::id::InstrumentId::from(inst.id as u32);
                                                            use audio_backend::effects::ReverbParameter as RP;
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: RP::Mix.as_index(),
                                                                    value: mx,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: RP::Decay.as_index(),
                                                                    value: dec,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: RP::RoomSize.as_index(),
                                                                    value: rs,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: RP::Damping.as_index(),
                                                                    value: damp,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: RP::Diffusion.as_index(),
                                                                    value: diff,
                                                                }
                                                                .into(),
                                                            );
                                                        }
                                                    }
                                                    if ui.button("Remove Reverb").clicked() {
                                                        to_remove_reverb = true;
                                                    }
                                                });
                                            });
                                        }
                                    }
                                    if to_remove_reverb {
                                        params
                                            .audio_effects
                                            .retain(|e| !matches!(e, AudioEffect::Reverb { .. }));
                                        rehydrate_ids.push(inst.id as u8);
                                    }

                                    if !has_reverb {
                                        ui.push_id(("add_hh_reverb", inst.id as u32), |ui| {
                                            if ui.button("Add Reverb").clicked() {
                                                params.audio_effects.push(AudioEffect::Reverb {
                                                    mix: 0.3,
                                                    decay_time: 0.6,
                                                    room_size: 1.0,
                                                    diffusion: 1.0,
                                                    damping: 0.2,
                                                });
                                                rehydrate_ids.push(inst.id as u8);
                                            }
                                        });
                                    }

                                    // Delay controls
                                    let mut has_delay = false;
                                    let mut to_remove_delay = false;
                                    for (eff_idx, eff) in params.audio_effects.iter_mut().enumerate() {
                                        if let AudioEffect::Delay {
                                            time,
                                            num_taps,
                                            feedback,
                                            mix,
                                        } = eff
                                        {
                                            has_delay = true;
                                            ui.push_id(("hh_delay", inst.id as u32), |ui| {
                                                egui::CollapsingHeader::new(format!(
                                                    "Delay {:02X}",
                                                    inst.id as u8
                                                ))
                                                .id_salt(("hh_delay_hdr", inst.id as u32))
                                                .show(ui, |ui| {
                                                    let mut changed = false;
                                                    let mut t = *time;
                                                    let mut tp = *num_taps;
                                                    let mut fb = *feedback;
                                                    let mut mx = *mix;
                                                    ui.horizontal(|ui| {
                                                        ui.label("Time (s)");
                                                        changed |= ui
                                                            .add(egui::Slider::new(
                                                                &mut t,
                                                                0.0..=audio_backend::effects::MAX_DELAY_SECONDS,
                                                            ))
                                                            .changed();
                                                        ui.label("Feedback");
                                                        changed |= ui
                                                            .add(egui::Slider::new(&mut fb, 0.0..=0.95))
                                                            .changed();
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Taps");
                                                        changed |= ui
                                                            .add(egui::Slider::new(
                                                                &mut tp,
                                                                1..=audio_backend::effects::MAX_TAPS as u8,
                                                            ))
                                                            .changed();
                                                        ui.label("Mix");
                                                        changed |= ui
                                                            .add(egui::Slider::new(&mut mx, 0.0..=1.0))
                                                            .changed();
                                                    });
                                                    if changed {
                                                        *time = t;
                                                        *num_taps = tp;
                                                        *feedback = fb;
                                                        *mix = mx;

                                                        if let Some(audio) = &mut audio_mgr.audio {
                                                            let id = audio_backend::id::InstrumentId::from(inst.id as u32);
                                                            use audio_backend::effects::DelayParameter as DP;
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: DP::Time.as_index(),
                                                                    value: t,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: DP::NumTaps.as_index(),
                                                                    value: tp as f32,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: DP::Feedback.as_index(),
                                                                    value: fb,
                                                                }
                                                                .into(),
                                                            );
                                                            audio.send_command(
                                                                audio_backend::MixerCmd::SetEffectParameter {
                                                                    instrument_id: id,
                                                                    effect_index: eff_idx,
                                                                    param_index: DP::Mix.as_index(),
                                                                    value: mx,
                                                                }
                                                                .into(),
                                                            );
                                                        }
                                                    }
                                                    if ui.button("Remove Delay").clicked() {
                                                        to_remove_delay = true;
                                                    }
                                                });
                                            });
                                        }
                                    }
                                    if to_remove_delay {
                                        params
                                            .audio_effects
                                            .retain(|e| !matches!(e, AudioEffect::Delay { .. }));
                                        rehydrate_ids.push(inst.id as u8);
                                    }

                                    if !has_delay {
                                        ui.push_id(("add_hh_delay", inst.id as u32), |ui| {
                                            if ui.button("Add Delay").clicked() {
                                                params.audio_effects.push(AudioEffect::Delay {
                                                    time: 0.3,
                                                    num_taps: 3,
                                                    feedback: 0.3,
                                                    mix: 0.35,
                                                });
                                                rehydrate_ids.push(inst.id as u8);
                                            }
                                        });
                                    }
                                }
                                InstrumentData::KickDrum(params) => {
                                    ui.label("Kick Drum");
                                    ui.separator();
                                    show_amp_envelope_controls(
                                        ui,
                                        &mut params.amp_envelope,
                                        inst.id,
                                        "kd",
                                        audio_mgr,
                                    );

                                    ui.separator();
                                    ui.label("Effects:");

                                    // Reverb controls
                                    let mut has_reverb = false;
                                    let mut to_remove_reverb = false;
                                    for (eff_idx, eff) in params.audio_effects.iter_mut().enumerate() {
                                        if let AudioEffect::Reverb { mix, decay_time, room_size, diffusion, damping } = eff {
                                            has_reverb = true;
                                            ui.push_id(("kd_reverb", inst.id as u32), |ui| {
                                                egui::CollapsingHeader::new(format!("Reverb {:02X}", inst.id as u8))
                                                    .id_salt(("kd_reverb_hdr", inst.id as u32))
                                                    .show(ui, |ui| {
                                                        let mut changed = false;
                                                        let mut mx = *mix;
                                                        let mut dec = *decay_time;
                                                        let mut rs = *room_size;
                                                        let mut diff = *diffusion;
                                                        let mut damp = *damping;
                                                        ui.horizontal(|ui| {
                                                            ui.label("Mix");
                                                            changed |= ui.add(egui::Slider::new(&mut mx, 0.0..=1.0)).changed();
                                                        });
                                                        ui.horizontal(|ui| {
                                                            ui.label("Decay");
                                                            changed |= ui.add(egui::Slider::new(&mut dec, 0.0..=1.0)).changed();
                                                            ui.label("Damping");
                                                            changed |= ui.add(egui::Slider::new(&mut damp, 0.0..=1.0)).changed();
                                                        });
                                                        ui.horizontal(|ui| {
                                                            ui.label("Room Size");
                                                            changed |= ui.add(egui::Slider::new(&mut rs, 0.5..=2.0)).changed();
                                                            ui.label("Diffusion");
                                                            changed |= ui.add(egui::Slider::new(&mut diff, 0.0..=1.0)).changed();
                                                        });
                                                        if changed {
                                                            *mix = mx;
                                                            *decay_time = dec;
                                                            *room_size = rs;
                                                            *diffusion = diff;
                                                            *damping = damp;

                                                            if let Some(audio) = &mut audio_mgr.audio {
                                                                let id = audio_backend::id::InstrumentId::from(inst.id as u32);
                                                                use audio_backend::effects::ReverbParameter as RP;
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::Mix.as_index(), value: mx }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::Decay.as_index(), value: dec }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::RoomSize.as_index(), value: rs }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::Damping.as_index(), value: damp }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::Diffusion.as_index(), value: diff }.into());
                                                            }
                                                        }
                                                        if ui.button("Remove Reverb").clicked() { to_remove_reverb = true; }
                                                    });
                                            });
                                        }
                                    }
                                    if to_remove_reverb {
                                        params.audio_effects.retain(|e| !matches!(e, AudioEffect::Reverb { .. }));
                                        rehydrate_ids.push(inst.id as u8);
                                    }

                                    if !has_reverb {
                                        ui.push_id(("add_kd_reverb", inst.id as u32), |ui| {
                                            if ui.button("Add Reverb").clicked() {
                                                params.audio_effects.push(AudioEffect::Reverb { mix: 0.25, decay_time: 0.4, room_size: 1.0, diffusion: 1.0, damping: 0.3 });
                                                rehydrate_ids.push(inst.id as u8);
                                            }
                                        });
                                    }

                                    // Delay controls
                                    let mut has_delay = false;
                                    let mut to_remove_delay = false;
                                    for (eff_idx, eff) in params.audio_effects.iter_mut().enumerate() {
                                        if let AudioEffect::Delay { time, num_taps, feedback, mix } = eff {
                                            has_delay = true;
                                            ui.push_id(("kd_delay", inst.id as u32), |ui| {
                                                egui::CollapsingHeader::new(format!("Delay {:02X}", inst.id as u8))
                                                    .id_salt(("kd_delay_hdr", inst.id as u32))
                                                    .show(ui, |ui| {
                                                        let mut changed = false;
                                                        let mut t = *time;
                                                        let mut tp = *num_taps;
                                                        let mut fb = *feedback;
                                                        let mut mx = *mix;
                                                        ui.horizontal(|ui| {
                                                            ui.label("Time (s)");
                                                            changed |= ui.add(egui::Slider::new(&mut t, 0.0..=audio_backend::effects::MAX_DELAY_SECONDS)).changed();
                                                            ui.label("Feedback");
                                                            changed |= ui.add(egui::Slider::new(&mut fb, 0.0..=0.95)).changed();
                                                        });
                                                        ui.horizontal(|ui| {
                                                            ui.label("Taps");
                                                            changed |= ui.add(egui::Slider::new(&mut tp, 1..=audio_backend::effects::MAX_TAPS as u8)).changed();
                                                            ui.label("Mix");
                                                            changed |= ui.add(egui::Slider::new(&mut mx, 0.0..=1.0)).changed();
                                                        });
                                                        if changed {
                                                            *time = t;
                                                            *num_taps = tp;
                                                            *feedback = fb;
                                                            *mix = mx;

                                                            if let Some(audio) = &mut audio_mgr.audio {
                                                                let id = audio_backend::id::InstrumentId::from(inst.id as u32);
                                                                use audio_backend::effects::DelayParameter as DP;
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: DP::Time.as_index(), value: t }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: DP::NumTaps.as_index(), value: tp as f32 }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: DP::Feedback.as_index(), value: fb }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: DP::Mix.as_index(), value: mx }.into());
                                                            }
                                                        }
                                                        if ui.button("Remove Delay").clicked() { to_remove_delay = true; }
                                                    });
                                            });
                                        }
                                    }
                                    if to_remove_delay {
                                        params.audio_effects.retain(|e| !matches!(e, AudioEffect::Delay { .. }));
                                        rehydrate_ids.push(inst.id as u8);
                                    }

                                    if !has_delay {
                                        ui.push_id(("add_kd_delay", inst.id as u32), |ui| {
                                            if ui.button("Add Delay").clicked() {
                                                params.audio_effects.push(AudioEffect::Delay { time: 0.2, num_taps: 2, feedback: 0.25, mix: 0.2 });
                                                rehydrate_ids.push(inst.id as u8);
                                            }
                                        });
                                    }
                                }
                                InstrumentData::SnareDrum(params) => {
                                    ui.label("Snare Drum");
                                    ui.separator();
                                    show_amp_envelope_controls(
                                        ui,
                                        &mut params.amp_envelope,
                                        inst.id,
                                        "sd",
                                        audio_mgr,
                                    );
                                    ui.separator();
                                    ui.label("Effects:");

                                    // Reverb controls
                                    let mut has_reverb = false;
                                    let mut to_remove_reverb = false;
                                    for (eff_idx, eff) in params.audio_effects.iter_mut().enumerate() {
                                        if let AudioEffect::Reverb { mix, decay_time, room_size, diffusion, damping } = eff {
                                            has_reverb = true;
                                            ui.push_id(("sd_reverb", inst.id as u32), |ui| {
                                                egui::CollapsingHeader::new(format!("Reverb {:02X}", inst.id as u8))
                                                    .id_salt(("sd_reverb_hdr", inst.id as u32))
                                                    .show(ui, |ui| {
                                                        let mut changed = false;
                                                        let mut mx = *mix;
                                                        let mut dec = *decay_time;
                                                        let mut rs = *room_size;
                                                        let mut diff = *diffusion;
                                                        let mut damp = *damping;
                                                        ui.horizontal(|ui| {
                                                            ui.label("Mix");
                                                            changed |= ui.add(egui::Slider::new(&mut mx, 0.0..=1.0)).changed();
                                                        });
                                                        ui.horizontal(|ui| {
                                                            ui.label("Decay");
                                                            changed |= ui.add(egui::Slider::new(&mut dec, 0.0..=1.0)).changed();
                                                            ui.label("Damping");
                                                            changed |= ui.add(egui::Slider::new(&mut damp, 0.0..=1.0)).changed();
                                                        });
                                                        ui.horizontal(|ui| {
                                                            ui.label("Room Size");
                                                            changed |= ui.add(egui::Slider::new(&mut rs, 0.5..=2.0)).changed();
                                                            ui.label("Diffusion");
                                                            changed |= ui.add(egui::Slider::new(&mut diff, 0.0..=1.0)).changed();
                                                        });
                                                        if changed {
                                                            *mix = mx;
                                                            *decay_time = dec;
                                                            *room_size = rs;
                                                            *diffusion = diff;
                                                            *damping = damp;

                                                            if let Some(audio) = &mut audio_mgr.audio {
                                                                let id = audio_backend::id::InstrumentId::from(inst.id as u32);
                                                                use audio_backend::effects::ReverbParameter as RP;
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::Mix.as_index(), value: mx }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::Decay.as_index(), value: dec }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::RoomSize.as_index(), value: rs }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::Damping.as_index(), value: damp }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::Diffusion.as_index(), value: diff }.into());
                                                            }
                                                        }
                                                        if ui.button("Remove Reverb").clicked() { to_remove_reverb = true; }
                                                    });
                                            });
                                        }
                                    }
                                    if to_remove_reverb {
                                        params.audio_effects.retain(|e| !matches!(e, AudioEffect::Reverb { .. }));
                                        rehydrate_ids.push(inst.id as u8);
                                    }

                                    if !has_reverb {
                                        ui.push_id(("add_sd_reverb", inst.id as u32), |ui| {
                                            if ui.button("Add Reverb").clicked() {
                                                params.audio_effects.push(AudioEffect::Reverb { mix: 0.25, decay_time: 0.4, room_size: 1.0, diffusion: 1.0, damping: 0.3 });
                                                rehydrate_ids.push(inst.id as u8);
                                            }
                                        });
                                    }

                                    // Delay controls
                                    let mut has_delay = false;
                                    let mut to_remove_delay = false;
                                    for (eff_idx, eff) in params.audio_effects.iter_mut().enumerate() {
                                        if let AudioEffect::Delay { time, num_taps, feedback, mix } = eff {
                                            has_delay = true;
                                            ui.push_id(("sd_delay", inst.id as u32), |ui| {
                                                egui::CollapsingHeader::new(format!("Delay {:02X}", inst.id as u8))
                                                    .id_salt(("sd_delay_hdr", inst.id as u32))
                                                    .show(ui, |ui| {
                                                        let mut changed = false;
                                                        let mut t = *time;
                                                        let mut tp = *num_taps;
                                                        let mut fb = *feedback;
                                                        let mut mx = *mix;
                                                        ui.horizontal(|ui| {
                                                            ui.label("Time (s)");
                                                            changed |= ui.add(egui::Slider::new(&mut t, 0.0..=audio_backend::effects::MAX_DELAY_SECONDS)).changed();
                                                            ui.label("Feedback");
                                                            changed |= ui.add(egui::Slider::new(&mut fb, 0.0..=0.95)).changed();
                                                        });
                                                        ui.horizontal(|ui| {
                                                            ui.label("Taps");
                                                            changed |= ui.add(egui::Slider::new(&mut tp, 1..=audio_backend::effects::MAX_TAPS as u8)).changed();
                                                            ui.label("Mix");
                                                            changed |= ui.add(egui::Slider::new(&mut mx, 0.0..=1.0)).changed();
                                                        });
                                                        if changed {
                                                            *time = t;
                                                            *num_taps = tp;
                                                            *feedback = fb;
                                                            *mix = mx;

                                                            if let Some(audio) = &mut audio_mgr.audio {
                                                                let id = audio_backend::id::InstrumentId::from(inst.id as u32);
                                                                use audio_backend::effects::DelayParameter as DP;
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: DP::Time.as_index(), value: t }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: DP::NumTaps.as_index(), value: tp as f32 }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: DP::Feedback.as_index(), value: fb }.into());
                                                                audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: DP::Mix.as_index(), value: mx }.into());
                                                            }
                                                        }
                                                        if ui.button("Remove Delay").clicked() { to_remove_delay = true; }
                                                    });
                                            });
                                        }
                                    }
                                    if to_remove_delay {
                                        params.audio_effects.retain(|e| !matches!(e, AudioEffect::Delay { .. }));
                                        rehydrate_ids.push(inst.id as u8);
                                    }

                                    if !has_delay {
                                        ui.push_id(("add_sd_delay", inst.id as u32), |ui| {
                                            if ui.button("Add Delay").clicked() {
                                                params.audio_effects.push(AudioEffect::Delay { time: 0.2, num_taps: 2, feedback: 0.25, mix: 0.2 });
                                                rehydrate_ids.push(inst.id as u8);
                                            }
                                        });
                                    }
                                }
                                InstrumentData::DFAM(params) => {
                                    ui.label("DFAM");
                                    ui.separator();
                                    show_amp_envelope_controls(
                                        ui,
                                        &mut params.amp_envelope,
                                        inst.id,
                                        "dfam",
                                        audio_mgr,
                                    );
                                    ui.separator();
                                    ui.label("Effects:");

                                    // Reverb controls
                                    let mut has_reverb = false;
                                    let mut to_remove_reverb = false;
                                    for (eff_idx, eff) in params.audio_effects.iter_mut().enumerate() {
                                        if let AudioEffect::Reverb { mix, decay_time, room_size, diffusion, damping } = eff {
                                            has_reverb = true;
                                            ui.push_id(("dfam_reverb", inst.id as u32), |ui| {
                                                egui::CollapsingHeader::new(format!("Reverb {:02X}", inst.id as u8))
                                                    .id_salt(("dfam_reverb_hdr", inst.id as u32))
                                                    .show(ui, |ui| {
                                                        let mut changed = false;
                                                        let mut mx = *mix;
                                                        let mut dec = *decay_time;
                                                        let mut rs = *room_size;
                                                        let mut diff = *diffusion;
                                                        let mut damp = *damping;
                                                        ui.horizontal(|ui| { ui.label("Mix"); changed |= ui.add(egui::Slider::new(&mut mx, 0.0..=1.0)).changed(); });
                                                        ui.horizontal(|ui| { ui.label("Decay"); changed |= ui.add(egui::Slider::new(&mut dec, 0.0..=1.0)).changed(); ui.label("Damping"); changed |= ui.add(egui::Slider::new(&mut damp, 0.0..=1.0)).changed(); });
                                                        ui.horizontal(|ui| { ui.label("Room Size"); changed |= ui.add(egui::Slider::new(&mut rs, 0.5..=2.0)).changed(); ui.label("Diffusion"); changed |= ui.add(egui::Slider::new(&mut diff, 0.0..=1.0)).changed(); });
                                                        if changed {
                                                            *mix = mx; *decay_time = dec; *room_size = rs; *diffusion = diff; *damping = damp;
                                                            if let Some(audio) = &mut audio_mgr.audio { let id = audio_backend::id::InstrumentId::from(inst.id as u32); use audio_backend::effects::ReverbParameter as RP; audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::Mix.as_index(), value: mx }.into()); audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::Decay.as_index(), value: dec }.into()); audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::RoomSize.as_index(), value: rs }.into()); audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::Damping.as_index(), value: damp }.into()); audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: RP::Diffusion.as_index(), value: diff }.into()); }
                                                        }
                                                        if ui.button("Remove Reverb").clicked() { to_remove_reverb = true; }
                                                    });
                                            });
                                        }
                                    }
                                    if to_remove_reverb { params.audio_effects.retain(|e| !matches!(e, AudioEffect::Reverb { .. })); rehydrate_ids.push(inst.id as u8); }
                                    if !has_reverb { ui.push_id(("add_dfam_reverb", inst.id as u32), |ui| { if ui.button("Add Reverb").clicked() { params.audio_effects.push(AudioEffect::Reverb { mix: 0.3, decay_time: 0.6, room_size: 1.0, diffusion: 1.0, damping: 0.2 }); rehydrate_ids.push(inst.id as u8); } }); }

                                    // Delay controls
                                    let mut has_delay = false;
                                    let mut to_remove_delay = false;
                                    for (eff_idx, eff) in params.audio_effects.iter_mut().enumerate() {
                                        if let AudioEffect::Delay { time, num_taps, feedback, mix } = eff {
                                            has_delay = true;
                                            ui.push_id(("dfam_delay", inst.id as u32), |ui| {
                                                egui::CollapsingHeader::new(format!("Delay {:02X}", inst.id as u8))
                                                    .id_salt(("dfam_delay_hdr", inst.id as u32))
                                                    .show(ui, |ui| {
                                                        let mut changed = false; let mut t = *time; let mut tp = *num_taps; let mut fb = *feedback; let mut mx = *mix;
                                                        ui.horizontal(|ui| { ui.label("Time (s)"); changed |= ui.add(egui::Slider::new(&mut t, 0.0..=audio_backend::effects::MAX_DELAY_SECONDS)).changed(); ui.label("Feedback"); changed |= ui.add(egui::Slider::new(&mut fb, 0.0..=0.95)).changed(); });
                                                        ui.horizontal(|ui| { ui.label("Taps"); changed |= ui.add(egui::Slider::new(&mut tp, 1..=audio_backend::effects::MAX_TAPS as u8)).changed(); ui.label("Mix"); changed |= ui.add(egui::Slider::new(&mut mx, 0.0..=1.0)).changed(); });
                                                        if changed { *time = t; *num_taps = tp; *feedback = fb; *mix = mx; if let Some(audio) = &mut audio_mgr.audio { let id = audio_backend::id::InstrumentId::from(inst.id as u32); use audio_backend::effects::DelayParameter as DP; audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: DP::Time.as_index(), value: t }.into()); audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: DP::NumTaps.as_index(), value: tp as f32 }.into()); audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: DP::Feedback.as_index(), value: fb }.into()); audio.send_command(audio_backend::MixerCmd::SetEffectParameter { instrument_id: id, effect_index: eff_idx, param_index: DP::Mix.as_index(), value: mx }.into()); } }
                                                        if ui.button("Remove Delay").clicked() { to_remove_delay = true; }
                                                    });
                                            });
                                        }
                                    }
                                    if to_remove_delay { params.audio_effects.retain(|e| !matches!(e, AudioEffect::Delay { .. })); rehydrate_ids.push(inst.id as u8); }
                                    if !has_delay { ui.push_id(("add_dfam_delay", inst.id as u32), |ui| { if ui.button("Add Delay").clicked() { params.audio_effects.push(AudioEffect::Delay { time: 0.3, num_taps: 3, feedback: 0.3, mix: 0.35 }); rehydrate_ids.push(inst.id as u8); } }); }
                                }
                                _ => {
                                    ui.label("Instrument editing not yet supported for this type.");
                                }
                            }
                        });
                    }
                }
            });
        if to_add_osc {
            let id = Self::next_free_instrument_id(song) as usize;
            song.instrument_bank.push(Instrument {
                id,
                name: format!("Osc {:02X}", id as u8),
                data: InstrumentData::SimpleOscillator(SimpleOscillatorParams {
                    waveform: Waveform::Sine,
                    audio_effects: Vec::new(),
                    amp_envelope: AmpEnvelopeParams {
                        attack: 0.01,
                        decay: 0.1,
                        sustain: 0.8,
                        release: 0.2,
                    },
                }),
            });
            // Create in backend with current params (no effects yet)
            if let InstrumentData::SimpleOscillator(ref params) =
                song.instrument_bank.last().unwrap().data
            {
                ensure_backend_osc_with_params(audio_mgr, id as u8, params);
            }
        }
        if to_add_hihat {
            let id = Self::next_free_instrument_id(song) as usize;
            song.instrument_bank.push(Instrument {
                id,
                name: format!("HiHat {:02X}", id as u8),
                data: InstrumentData::HiHat(HiHatParams {
                    audio_effects: Vec::new(),
                    amp_envelope: AmpEnvelopeParams {
                        attack: 0.001,
                        decay: 0.08,
                        sustain: 0.0,
                        release: 0.15,
                    },
                }),
            });
            if let InstrumentData::HiHat(ref params) = song.instrument_bank.last().unwrap().data {
                ensure_backend_hihat_with_params(audio_mgr, id as u8, params);
            }
        }
        if to_add_kick {
            let id = Self::next_free_instrument_id(song) as usize;
            song.instrument_bank.push(Instrument {
                id,
                name: format!("Kick {:02X}", id as u8),
                data: InstrumentData::KickDrum(KickDrumParams {
                    audio_effects: Vec::new(),
                    amp_envelope: sequencer::models::AmpEnvelopeParams {
                        attack: 0.01,
                        decay: 0.1,
                        sustain: 0.0,
                        release: 0.1,
                    },
                    pitch_envelope: sequencer::models::PitchEnvelopeParams {
                        freq_delta: -100.0,
                        decay_time: 0.05,
                    },
                }),
            });
            if let InstrumentData::KickDrum(ref params) = song.instrument_bank.last().unwrap().data
            {
                ensure_backend_kick_with_params(audio_mgr, id as u8, params);
            }
        }
        if to_add_snare {
            let id = Self::next_free_instrument_id(song) as usize;
            song.instrument_bank.push(Instrument {
                id,
                name: format!("Snare {:02X}", id as u8),
                data: InstrumentData::SnareDrum(SnareDrumParams {
                    audio_effects: Vec::new(),
                    amp_envelope: AmpEnvelopeParams {
                        attack: 0.005,
                        decay: 0.2,
                        sustain: 0.0,
                        release: 0.3,
                    },
                }),
            });
            if let InstrumentData::SnareDrum(ref params) = song.instrument_bank.last().unwrap().data
            {
                ensure_backend_snare_with_params(audio_mgr, id as u8, params);
            }
        }
        if to_add_dfam {
            let id = Self::next_free_instrument_id(song) as usize;
            song.instrument_bank.push(Instrument {
                id,
                name: format!("DFAM {:02X}", id as u8),
                data: InstrumentData::DFAM(sequencer::models::DFAMParams {
                    audio_effects: Vec::new(),
                    amp_envelope: AmpEnvelopeParams {
                        attack: 0.001,
                        decay: 0.08,
                        sustain: 0.0,
                        release: 0.15,
                    },
                }),
            });
            if let InstrumentData::DFAM(ref params) = song.instrument_bank.last().unwrap().data {
                ensure_backend_dfam_with_params(audio_mgr, id as u8, params);
            }
        }
        // Apply updates to backend after UI draw
        rehydrate_ids.sort();
        rehydrate_ids.dedup();
        for id_u8 in rehydrate_ids {
            if let Some(inst) = song.instrument_bank.iter().find(|i| i.id as u8 == id_u8) {
                match &inst.data {
                    InstrumentData::SimpleOscillator(params) => {
                        ensure_backend_osc_with_params(audio_mgr, id_u8, params);
                    }
                    InstrumentData::HiHat(params) => {
                        ensure_backend_hihat_with_params(audio_mgr, id_u8, params);
                    }
                    InstrumentData::KickDrum(params) => {
                        ensure_backend_kick_with_params(audio_mgr, id_u8, params);
                    }
                    InstrumentData::SnareDrum(params) => {
                        ensure_backend_snare_with_params(audio_mgr, id_u8, params);
                    }
                    InstrumentData::DFAM(params) => {
                        ensure_backend_dfam_with_params(audio_mgr, id_u8, params);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Create/replace the backend DFAM instrument and configure its voice effects from params.
fn ensure_backend_dfam_with_params(
    audio_mgr: &mut AudioManager,
    id_u8: u8,
    params: &sequencer::models::DFAMParams,
) {
    if let Some(audio) = &mut audio_mgr.audio {
        let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
        let instrument = audio.get_instrument_factory().create_dfam(id, 0.0);
        audio.send_command(audio_backend::SequencerCmd::AddTrackInstrument { instrument }.into());

        // Always add a default Moog Ladder filter with cutoff 500 Hz & resonance 0.5
        let ladder = audio.get_effect_factory().create_moog_ladder(500.0, 0.5);
        audio.send_command(
            audio_backend::SequencerCmd::AddEffectToInstrument {
                instrument_id: id,
                effect: ladder,
            }
            .into(),
        );

        // Configure any additional mono insert effects from params (reverb/delay)
        for eff in &params.audio_effects {
            match eff {
                AudioEffect::Reverb {
                    mix,
                    decay_time,
                    room_size,
                    diffusion,
                    damping,
                } => {
                    let mut r = audio.get_effect_factory().create_mono_reverb();
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Mix.as_index(),
                        (*mix).clamp(0.0, 1.0),
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Decay.as_index(),
                        *decay_time,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::RoomSize.as_index(),
                        *room_size,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Damping.as_index(),
                        *damping,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *r,
                        audio_backend::effects::ReverbParameter::Diffusion.as_index(),
                        *diffusion,
                    );
                    audio.send_command(
                        audio_backend::SequencerCmd::AddEffectToInstrument {
                            instrument_id: id,
                            effect: r,
                        }
                        .into(),
                    );
                }
                AudioEffect::Delay {
                    time,
                    num_taps,
                    feedback,
                    mix,
                } => {
                    let mut d = audio.get_effect_factory().create_mono_delay(
                        *time,
                        *num_taps as usize,
                        *feedback,
                        *mix,
                    );
                    use audio_backend::effects::DelayParameter as DP;
                    audio_backend::MonoEffect::set_parameter(&mut *d, DP::Time.as_index(), *time);
                    audio_backend::MonoEffect::set_parameter(
                        &mut *d,
                        DP::NumTaps.as_index(),
                        *num_taps as f32,
                    );
                    audio_backend::MonoEffect::set_parameter(
                        &mut *d,
                        DP::Feedback.as_index(),
                        *feedback,
                    );
                    audio_backend::MonoEffect::set_parameter(&mut *d, DP::Mix.as_index(), *mix);
                    audio.send_command(
                        audio_backend::SequencerCmd::AddEffectToInstrument {
                            instrument_id: id,
                            effect: d,
                        }
                        .into(),
                    );
                }
            }
        }
    }

    send_amp_envelope_to_backend(audio_mgr, id_u8, &params.amp_envelope);
}
