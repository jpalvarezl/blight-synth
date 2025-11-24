use eframe::egui;
use sequencer::models::{
    AmpEnvelopeParams, HiHatParams, Instrument, InstrumentData, KickDrumParams,
    SimpleOscillatorParams, SnareDrumParams, Song, Waveform,
};

use crate::audio::AudioManager;
use crate::ui_components::effect_controls::{
    DelayDefaults, EffectPanelConfig, ReverbDefaults, show_effect_panels,
};

mod backend;
use backend::{ensure_backend_instrument, send_amp_envelope_to_backend};

#[derive(Default)]
pub struct InstrumentManagerWindow {
    pub open: bool,
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
                                    show_effect_panels(
                                        ui,
                                        EffectPanelConfig {
                                            instrument_id: inst.id,
                                            ui_prefix: "osc",
                                            reverb_defaults: Some(ReverbDefaults::new(
                                                0.3, 0.6, 1.0, 1.0, 0.2,
                                            )),
                                            delay_defaults: Some(DelayDefaults::new(
                                                0.3, 3, 0.3, 0.35,
                                            )),
                                        },
                                        &mut params.audio_effects,
                                        audio_mgr,
                                        &mut rehydrate_ids,
                                    );
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
                                    show_effect_panels(
                                        ui,
                                        EffectPanelConfig {
                                            instrument_id: inst.id,
                                            ui_prefix: "hh",
                                            reverb_defaults: Some(ReverbDefaults::new(
                                                0.3, 0.6, 1.0, 1.0, 0.2,
                                            )),
                                            delay_defaults: Some(DelayDefaults::new(
                                                0.3, 3, 0.3, 0.35,
                                            )),
                                        },
                                        &mut params.audio_effects,
                                        audio_mgr,
                                        &mut rehydrate_ids,
                                    );
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
                                    show_effect_panels(
                                        ui,
                                        EffectPanelConfig {
                                            instrument_id: inst.id,
                                            ui_prefix: "kd",
                                            reverb_defaults: Some(ReverbDefaults::new(
                                                0.25, 0.4, 1.0, 1.0, 0.3,
                                            )),
                                            delay_defaults: Some(DelayDefaults::new(
                                                0.2, 2, 0.25, 0.2,
                                            )),
                                        },
                                        &mut params.audio_effects,
                                        audio_mgr,
                                        &mut rehydrate_ids,
                                    );
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
                                    show_effect_panels(
                                        ui,
                                        EffectPanelConfig {
                                            instrument_id: inst.id,
                                            ui_prefix: "sd",
                                            reverb_defaults: Some(ReverbDefaults::new(
                                                0.25, 0.4, 1.0, 1.0, 0.3,
                                            )),
                                            delay_defaults: Some(DelayDefaults::new(
                                                0.2, 2, 0.25, 0.2,
                                            )),
                                        },
                                        &mut params.audio_effects,
                                        audio_mgr,
                                        &mut rehydrate_ids,
                                    );
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
                                    show_effect_panels(
                                        ui,
                                        EffectPanelConfig {
                                            instrument_id: inst.id,
                                            ui_prefix: "dfam",
                                            reverb_defaults: Some(ReverbDefaults::new(
                                                0.3, 0.6, 1.0, 1.0, 0.2,
                                            )),
                                            delay_defaults: Some(DelayDefaults::new(
                                                0.3, 3, 0.3, 0.35,
                                            )),
                                        },
                                        &mut params.audio_effects,
                                        audio_mgr,
                                        &mut rehydrate_ids,
                                    );
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
            if let Some(inst) = song.instrument_bank.last() {
                ensure_backend_instrument(audio_mgr, inst.id as u8, &inst.data);
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
            if let Some(inst) = song.instrument_bank.last() {
                ensure_backend_instrument(audio_mgr, inst.id as u8, &inst.data);
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
            if let Some(inst) = song.instrument_bank.last() {
                ensure_backend_instrument(audio_mgr, inst.id as u8, &inst.data);
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
            if let Some(inst) = song.instrument_bank.last() {
                ensure_backend_instrument(audio_mgr, inst.id as u8, &inst.data);
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
            if let Some(inst) = song.instrument_bank.last() {
                ensure_backend_instrument(audio_mgr, inst.id as u8, &inst.data);
            }
        }
        // Apply updates to backend after UI draw
        rehydrate_ids.sort();
        rehydrate_ids.dedup();
        for id_u8 in rehydrate_ids {
            if let Some(inst) = song.instrument_bank.iter().find(|i| i.id as u8 == id_u8) {
                ensure_backend_instrument(audio_mgr, id_u8, &inst.data);
            }
        }
    }
}
