use eframe::egui;
use sequencer::models::{AudioEffect, Instrument, InstrumentData, SimpleOscillatorParams, Song, Waveform};

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
                AudioEffect::Reverb { wet_mix, dry_mix, feedback, damping } => {
                    let mut r = audio.get_effect_factory().create_mono_reverb();
                    // Compute wet/dry ratio
                    let total = (*wet_mix + *dry_mix).max(1e-6);
                    let wet_ratio = (*wet_mix / total).clamp(0.0, 1.0);
                    // Set parameters on the effect (MonoEffect trait is in audio_backend)
                    audio_backend::MonoEffect::set_parameter(&mut *r, 0, 1.0); // wet level
                    audio_backend::MonoEffect::set_parameter(&mut *r, 1, wet_ratio);
                    audio_backend::MonoEffect::set_parameter(&mut *r, 2, feedback.clamp(0.0, 0.99));
                    audio_backend::MonoEffect::set_parameter(&mut *r, 3, damping.clamp(0.0, 1.0));

                    audio.send_command(
                        audio_backend::SequencerCmd::AddEffectToInstrument {
                            instrument_id: id,
                            effect: r,
                        }
                        .into(),
                    );
                }
                _ => {}
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
                                    for eff in params.audio_effects.iter_mut() {
                                        if let AudioEffect::Reverb { wet_mix, dry_mix, feedback, damping } = eff {
                                            has_reverb = true;
                                            // Namespace all inner widgets by instrument id to avoid ID clashes
                                            ui.push_id(("reverb", inst.id as u32), |ui| {
                                                egui::CollapsingHeader::new(format!("Reverb {:02X}", inst.id as u8))
                                                    .id_source(("reverb_hdr", inst.id as u32))
                                                    .show(ui, |ui| {
                                                        let mut changed = false;
                                                        let mut w = *wet_mix;
                                                        let mut d = *dry_mix;
                                                        let mut fb = *feedback;
                                                        let mut dp = *damping;
                                                        ui.horizontal(|ui| {
                                                            ui.label("Wet");
                                                            changed |= ui.add(egui::Slider::new(&mut w, 0.0..=1.0)).changed();
                                                            ui.label("Dry");
                                                            changed |= ui.add(egui::Slider::new(&mut d, 0.0..=1.0)).changed();
                                                        });
                                                        ui.horizontal(|ui| {
                                                            ui.label("Feedback");
                                                            changed |= ui.add(egui::Slider::new(&mut fb, 0.0..=0.99)).changed();
                                                            ui.label("Damping");
                                                            changed |= ui.add(egui::Slider::new(&mut dp, 0.0..=1.0)).changed();
                                                        });
                                                        if changed {
                                                            *wet_mix = w;
                                                            *dry_mix = d;
                                                            *feedback = fb;
                                                            *damping = dp;
                                                            rehydrate_ids.push(inst.id as u8);
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
                                                    wet_mix: 0.3,
                                                    dry_mix: 0.7,
                                                    feedback: 0.6,
                                                    damping: 0.2,
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
            if let InstrumentData::SimpleOscillator(ref params) = song.instrument_bank.last().unwrap().data {
                ensure_backend_osc_with_params(audio_mgr, id as u8, params);
            }
        }
        // Apply updates to backend after UI draw
        rehydrate_ids.sort();
        rehydrate_ids.dedup();
        for id_u8 in rehydrate_ids {
            if let Some(inst) = song.instrument_bank.iter().find(|i| i.id as u8 == id_u8) {
                if let InstrumentData::SimpleOscillator(ref params) = inst.data {
                    ensure_backend_osc_with_params(audio_mgr, id_u8, params);
                }
            }
        }
    }
}
