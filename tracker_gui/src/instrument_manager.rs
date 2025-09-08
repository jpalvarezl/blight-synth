use eframe::egui;
use sequencer::models::{
    AudioEffect, Instrument, InstrumentData, SimpleOscillatorParams, Song, Waveform, HiHatParams,
};

use crate::audio::AudioManager;
use crate::audio_utils::map_waveform_to_backend;

#[derive(Default)]
pub struct InstrumentManagerWindow {
    pub open: bool,
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
}

/// Create/replace the backend hi-hat instrument and configure its voice effects from params.
fn ensure_backend_hihat_with_params(
    audio_mgr: &mut AudioManager,
    id_u8: u8,
    params: &HiHatParams,
) {
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
                }),
            });
            if let InstrumentData::HiHat(ref params) = song.instrument_bank.last().unwrap().data {
                ensure_backend_hihat_with_params(audio_mgr, id as u8, params);
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
                    _ => {}
                }
            }
        }
    }
}
