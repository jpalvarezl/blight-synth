use crate::audio::{AudioManager, TRACKER_EFFECT_ID};
use audio_backend::effects::{
    DelayParameter as DP, MAX_DELAY_SECONDS, MAX_TAPS, ReverbParameter as RP,
};
use eframe::egui;
use sequencer::models::AudioEffect;

pub trait EffectSync {
    fn queue_rehydrate(&mut self, instrument_id: u8);
}

pub struct EffectPanelConfig {
    pub instrument_id: usize,
    pub ui_prefix: &'static str,
    pub reverb_defaults: Option<ReverbDefaults>,
    pub delay_defaults: Option<DelayDefaults>,
}

#[derive(Clone, Copy)]
pub struct ReverbDefaults {
    pub mix: f32,
    pub decay_time: f32,
    pub room_size: f32,
    pub diffusion: f32,
    pub damping: f32,
}

impl ReverbDefaults {
    pub const fn new(
        mix: f32,
        decay_time: f32,
        room_size: f32,
        diffusion: f32,
        damping: f32,
    ) -> Self {
        Self {
            mix,
            decay_time,
            room_size,
            diffusion,
            damping,
        }
    }
}

#[derive(Clone, Copy)]
pub struct DelayDefaults {
    pub time: f32,
    pub num_taps: u8,
    pub feedback: f32,
    pub mix: f32,
}

impl DelayDefaults {
    pub const fn new(time: f32, num_taps: u8, feedback: f32, mix: f32) -> Self {
        Self {
            time,
            num_taps,
            feedback,
            mix,
        }
    }
}

pub fn show_effect_panels(
    ui: &mut egui::Ui,
    config: EffectPanelConfig,
    effects: &mut Vec<AudioEffect>,
    audio_mgr: &mut AudioManager,
    sync: &mut dyn EffectSync,
) {
    if let Some(defaults) = config.reverb_defaults {
        show_reverb_section(ui, &config, defaults, effects, audio_mgr, sync);
    }
    if let Some(defaults) = config.delay_defaults {
        show_delay_section(ui, &config, defaults, effects, audio_mgr, sync);
    }
}

fn show_reverb_section(
    ui: &mut egui::Ui,
    config: &EffectPanelConfig,
    defaults: ReverbDefaults,
    effects: &mut Vec<AudioEffect>,
    audio_mgr: &mut AudioManager,
    sync: &mut dyn EffectSync,
) {
    let inst_id_u8 = config.instrument_id as u8;
    let reverb_idx = effects
        .iter()
        .position(|eff| matches!(eff, AudioEffect::Reverb { .. }));

    if let Some(idx) = reverb_idx {
        let mut remove_reverb = false;
        if let AudioEffect::Reverb {
            mix,
            decay_time,
            room_size,
            diffusion,
            damping,
        } = &mut effects[idx]
        {
            ui.push_id(
                (config.ui_prefix, config.instrument_id as u32, "reverb"),
                |ui| {
                    egui::CollapsingHeader::new(format!("Reverb {:02X}", inst_id_u8))
                        .id_salt((config.ui_prefix, config.instrument_id as u32, "reverb_hdr"))
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
                                changed |=
                                    ui.add(egui::Slider::new(&mut damp, 0.0..=1.0)).changed();
                            });
                            ui.horizontal(|ui| {
                                ui.label("Room Size");
                                changed |= ui.add(egui::Slider::new(&mut rs, 0.5..=2.0)).changed();
                                ui.label("Diffusion");
                                changed |=
                                    ui.add(egui::Slider::new(&mut diff, 0.0..=1.0)).changed();
                            });

                            if changed {
                                *mix = mx;
                                *decay_time = dec;
                                *room_size = rs;
                                *diffusion = diff;
                                *damping = damp;
                                push_reverb_updates(
                                    audio_mgr,
                                    config.instrument_id,
                                    mx,
                                    dec,
                                    rs,
                                    damp,
                                    diff,
                                );
                            }

                            if ui.button("Remove Reverb").clicked() {
                                remove_reverb = true;
                            }
                        });
                },
            );
        }
        if remove_reverb {
            effects.remove(idx);
            sync.queue_rehydrate(inst_id_u8);
        }
    } else {
        let should_add = ui
            .push_id(
                (config.ui_prefix, config.instrument_id as u32, "add_reverb"),
                |ui| ui.button("Add Reverb").clicked(),
            )
            .inner;
        if should_add {
            effects.push(AudioEffect::Reverb {
                mix: defaults.mix,
                decay_time: defaults.decay_time,
                room_size: defaults.room_size,
                diffusion: defaults.diffusion,
                damping: defaults.damping,
            });
            sync.queue_rehydrate(inst_id_u8);
        }
    }
}

fn show_delay_section(
    ui: &mut egui::Ui,
    config: &EffectPanelConfig,
    defaults: DelayDefaults,
    effects: &mut Vec<AudioEffect>,
    audio_mgr: &mut AudioManager,
    sync: &mut dyn EffectSync,
) {
    let inst_id_u8 = config.instrument_id as u8;
    let delay_idx = effects
        .iter()
        .position(|eff| matches!(eff, AudioEffect::Delay { .. }));

    if let Some(idx) = delay_idx {
        let mut remove_delay = false;
        if let AudioEffect::Delay {
            time,
            num_taps,
            feedback,
            mix,
        } = &mut effects[idx]
        {
            ui.push_id(
                (config.ui_prefix, config.instrument_id as u32, "delay"),
                |ui| {
                    egui::CollapsingHeader::new(format!("Delay {:02X}", inst_id_u8))
                        .id_salt((config.ui_prefix, config.instrument_id as u32, "delay_hdr"))
                        .show(ui, |ui| {
                            let mut changed = false;
                            let mut t = *time;
                            let mut taps = *num_taps;
                            let mut fb = *feedback;
                            let mut mx = *mix;

                            ui.horizontal(|ui| {
                                ui.label("Time (s)");
                                changed |= ui
                                    .add(egui::Slider::new(&mut t, 0.0..=MAX_DELAY_SECONDS))
                                    .changed();
                                ui.label("Feedback");
                                changed |= ui.add(egui::Slider::new(&mut fb, 0.0..=0.95)).changed();
                            });
                            ui.horizontal(|ui| {
                                ui.label("Taps");
                                changed |= ui
                                    .add(egui::Slider::new(&mut taps, 1..=MAX_TAPS as u8))
                                    .changed();
                                ui.label("Mix");
                                changed |= ui.add(egui::Slider::new(&mut mx, 0.0..=1.0)).changed();
                            });

                            if changed {
                                *time = t;
                                *num_taps = taps;
                                *feedback = fb;
                                *mix = mx;
                                push_delay_updates(
                                    audio_mgr,
                                    config.instrument_id,
                                    t,
                                    taps,
                                    fb,
                                    mx,
                                );
                            }

                            if ui.button("Remove Delay").clicked() {
                                remove_delay = true;
                            }
                        });
                },
            );
        }
        if remove_delay {
            effects.remove(idx);
            sync.queue_rehydrate(inst_id_u8);
        }
    } else {
        let should_add = ui
            .push_id(
                (config.ui_prefix, config.instrument_id as u32, "add_delay"),
                |ui| ui.button("Add Delay").clicked(),
            )
            .inner;
        if should_add {
            effects.push(AudioEffect::Delay {
                time: defaults.time,
                num_taps: defaults.num_taps,
                feedback: defaults.feedback,
                mix: defaults.mix,
            });
            sync.queue_rehydrate(inst_id_u8);
        }
    }
}

fn push_reverb_updates(
    audio_mgr: &mut AudioManager,
    instrument_id: usize,
    mix: f32,
    decay: f32,
    room_size: f32,
    damping: f32,
    diffusion: f32,
) {
    if let Some(audio) = &mut audio_mgr.audio {
        let id = audio_backend::id::InstrumentId::from(instrument_id as u32);
        for (param_index, value) in [
            (RP::Mix.as_index(), mix),
            (RP::Decay.as_index(), decay),
            (RP::RoomSize.as_index(), room_size),
            (RP::Damping.as_index(), damping),
            (RP::Diffusion.as_index(), diffusion),
        ] {
            audio.send_command(
                audio_backend::MixerCmd::SetEffectParameter {
                    instrument_id: id,
                    effect_id: TRACKER_EFFECT_ID,
                    param_index,
                    value,
                }
                .into(),
            );
        }
    }
}

fn push_delay_updates(
    audio_mgr: &mut AudioManager,
    instrument_id: usize,
    time: f32,
    num_taps: u8,
    feedback: f32,
    mix: f32,
) {
    if let Some(audio) = &mut audio_mgr.audio {
        let id = audio_backend::id::InstrumentId::from(instrument_id as u32);
        for (param_index, value) in [
            (DP::Time.as_index(), time),
            (DP::NumTaps.as_index(), num_taps as f32),
            (DP::Feedback.as_index(), feedback),
            (DP::Mix.as_index(), mix),
        ] {
            audio.send_command(
                audio_backend::MixerCmd::SetEffectParameter {
                    instrument_id: id,
                    effect_id: TRACKER_EFFECT_ID,
                    param_index,
                    value,
                }
                .into(),
            );
        }
    }
}
