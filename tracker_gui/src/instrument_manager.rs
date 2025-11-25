use eframe::egui;
use sequencer::models::{
    AmpEnvelopeParams, HiHatParams, Instrument, InstrumentData, KickDrumParams,
    SimpleOscillatorParams, SnareDrumParams, Song, Waveform,
};

use crate::audio::AudioManager;
use crate::ui_components::{
    show_amp_envelope_editor, show_effect_panels, DelayDefaults, EffectPanelConfig,
    ReverbDefaults,
};

pub mod backend;
use backend::{ensure_backend_instrument, send_amp_envelope_to_backend};
mod sync;
use sync::InstrumentSync;

struct InstrumentUiMetadata {
    label: &'static str,
    ui_prefix: &'static str,
    reverb_defaults: Option<ReverbDefaults>,
    delay_defaults: Option<DelayDefaults>,
}

impl InstrumentUiMetadata {
    const fn new(
        label: &'static str,
        ui_prefix: &'static str,
        reverb_defaults: Option<ReverbDefaults>,
        delay_defaults: Option<DelayDefaults>,
    ) -> Self {
        Self {
            label,
            ui_prefix,
            reverb_defaults,
            delay_defaults,
        }
    }
}

const OSCILLATOR_UI: InstrumentUiMetadata = InstrumentUiMetadata::new(
    "Oscillator",
    "osc",
    Some(ReverbDefaults::new(0.3, 0.6, 1.0, 1.0, 0.2)),
    Some(DelayDefaults::new(0.3, 3, 0.3, 0.35)),
);
const HIHAT_UI: InstrumentUiMetadata = InstrumentUiMetadata::new(
    "Hi-Hat",
    "hh",
    Some(ReverbDefaults::new(0.3, 0.6, 1.0, 1.0, 0.2)),
    Some(DelayDefaults::new(0.3, 3, 0.3, 0.35)),
);
const KICK_UI: InstrumentUiMetadata = InstrumentUiMetadata::new(
    "Kick Drum",
    "kd",
    Some(ReverbDefaults::new(0.25, 0.4, 1.0, 1.0, 0.3)),
    Some(DelayDefaults::new(0.2, 2, 0.25, 0.2)),
);
const SNARE_UI: InstrumentUiMetadata = InstrumentUiMetadata::new(
    "Snare Drum",
    "sd",
    Some(ReverbDefaults::new(0.25, 0.4, 1.0, 1.0, 0.3)),
    Some(DelayDefaults::new(0.2, 2, 0.25, 0.2)),
);
const DFAM_UI: InstrumentUiMetadata = InstrumentUiMetadata::new(
    "DFAM",
    "dfam",
    Some(ReverbDefaults::new(0.3, 0.6, 1.0, 1.0, 0.2)),
    Some(DelayDefaults::new(0.3, 3, 0.3, 0.35)),
);

fn show_envelope_and_effects(
    ui: &mut egui::Ui,
    meta: &InstrumentUiMetadata,
    inst_id: usize,
    amp_env: &mut AmpEnvelopeParams,
    effects: &mut Vec<sequencer::models::AudioEffect>,
    audio_mgr: &mut AudioManager,
    sync: &mut InstrumentSync,
) {
    show_amp_envelope_editor(ui, amp_env, inst_id, meta.ui_prefix, |env| {
        send_amp_envelope_to_backend(audio_mgr, inst_id as u8, env)
    });
    ui.separator();
    ui.label("Effects:");
    show_effect_panels(
        ui,
        EffectPanelConfig {
            instrument_id: inst_id,
            ui_prefix: meta.ui_prefix,
            reverb_defaults: meta.reverb_defaults,
            delay_defaults: meta.delay_defaults,
        },
        effects,
        audio_mgr,
        sync,
    );
}

#[derive(Default)]
pub struct InstrumentManagerWindow {
    pub open: bool,
    sync: InstrumentSync,
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
                                    let meta = &OSCILLATOR_UI;
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
                                            self.sync.queue_rehydrate(inst.id as u8);
                                        }
                                    });
                                    ui.separator();
                                    show_envelope_and_effects(
                                        ui,
                                        meta,
                                        inst.id,
                                        &mut params.amp_envelope,
                                        &mut params.audio_effects,
                                        audio_mgr,
                                        &mut self.sync,
                                    );
                                }
                                InstrumentData::HiHat(params) => {
                                    let meta = &HIHAT_UI;
                                    ui.label(meta.label);
                                    ui.separator();
                                    show_envelope_and_effects(
                                        ui,
                                        meta,
                                        inst.id,
                                        &mut params.amp_envelope,
                                        &mut params.audio_effects,
                                        audio_mgr,
                                        &mut self.sync,
                                    );
                                }
                                InstrumentData::KickDrum(params) => {
                                    let meta = &KICK_UI;
                                    ui.label(meta.label);
                                    ui.separator();
                                    show_envelope_and_effects(
                                        ui,
                                        meta,
                                        inst.id,
                                        &mut params.amp_envelope,
                                        &mut params.audio_effects,
                                        audio_mgr,
                                        &mut self.sync,
                                    );
                                }
                                InstrumentData::SnareDrum(params) => {
                                    let meta = &SNARE_UI;
                                    ui.label(meta.label);
                                    ui.separator();
                                    show_envelope_and_effects(
                                        ui,
                                        meta,
                                        inst.id,
                                        &mut params.amp_envelope,
                                        &mut params.audio_effects,
                                        audio_mgr,
                                        &mut self.sync,
                                    );
                                }
                                InstrumentData::DFAM(params) => {
                                    let meta = &DFAM_UI;
                                    ui.label(meta.label);
                                    ui.separator();
                                    show_envelope_and_effects(
                                        ui,
                                        meta,
                                        inst.id,
                                        &mut params.amp_envelope,
                                        &mut params.audio_effects,
                                        audio_mgr,
                                        &mut self.sync,
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
        self.sync.apply_pending(song, audio_mgr);
    }
}
