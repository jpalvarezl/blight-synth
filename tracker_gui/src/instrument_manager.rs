use eframe::egui;
use sequencer::models::{Instrument, InstrumentData, SimpleOscillatorParams, Song, Waveform};

use crate::audio::AudioManager;
use crate::audio_utils::map_waveform_to_backend;

#[derive(Default)]
pub struct InstrumentManagerWindow {
    pub open: bool,
}

fn ensure_backend_osc(audio_mgr: &mut AudioManager, id_u8: u8, wf: Waveform) {
    if let Some(audio) = &mut audio_mgr.audio {
        let backend_wave = map_waveform_to_backend(wf);
        let id = audio_backend::id::InstrumentId::from(id_u8 as u32);
        let instrument = audio
            .get_instrument_factory()
            .create_oscillator_with_waveform(id, 0.0, backend_wave);
        audio.send_command(audio_backend::SequencerCmd::AddTrackInstrument { instrument }.into());
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
                                            .selected_text(match wf {
                                                Waveform::Sine => "Sine",
                                                Waveform::Square => "Square",
                                                Waveform::Sawtooth => "Sawtooth",
                                                Waveform::Triangle => "Triangle",
                                                Waveform::NesTriangle => "NES Triangle",
                                            })
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
                                                            match w {
                                                                Waveform::Sine => "Sine",
                                                                Waveform::Square => "Square",
                                                                Waveform::Sawtooth => "Sawtooth",
                                                                Waveform::Triangle => "Triangle",
                                                                Waveform::NesTriangle => {
                                                                    "NES Triangle"
                                                                }
                                                            },
                                                        )
                                                        .clicked()
                                                    {
                                                        wf = w;
                                                    }
                                                }
                                            });
                                        if wf != params.waveform {
                                            params.waveform = wf;
                                            // defer backend update after UI block to avoid borrow issues
                                        }
                                    });
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
                }),
            });
            ensure_backend_osc(audio_mgr, id as u8, Waveform::Sine);
        }
    }
}
