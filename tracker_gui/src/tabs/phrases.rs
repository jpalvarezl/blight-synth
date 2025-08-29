use crate::ui_components::hex_u8_editor;
use crate::ui_state::UiState;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use sequencer::models::{Phrase, Song};

#[derive(Default)]
pub struct PhrasesTab {
    pub selected_phrase: usize,
    pub selected_event_step: Option<usize>,
}

impl PhrasesTab {
    pub fn reset(&mut self) {
        self.selected_phrase = 0;
        self.selected_event_step = None;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, song: &mut Song, ui_state: &mut UiState) {
        ui.horizontal(|ui| {
            ui.label(format!("Phrases: {} total", song.phrase_bank.len()));
            if ui.button("Add Phrase").clicked() {
                song.phrase_bank.push(Phrase::default());
            }
            if ui.button("Remove Phrase").clicked() && !song.phrase_bank.is_empty() {
                if self.selected_phrase < song.phrase_bank.len() {
                    song.phrase_bank.remove(self.selected_phrase);
                    if self.selected_phrase >= song.phrase_bank.len()
                        && !song.phrase_bank.is_empty()
                    {
                        self.selected_phrase = song.phrase_bank.len() - 1;
                    }
                }
            }
        });

        ui.separator();

        if song.phrase_bank.is_empty() {
            ui.label("No phrases available. Click 'Add Phrase' to create one.");
            return;
        }

        // Phrase selector
        ui.horizontal(|ui| {
            ui.label("Phrase:");
            for (i, _) in song.phrase_bank.iter().enumerate() {
                if ui
                    .selectable_label(i == self.selected_phrase, format!("{:02X}", i))
                    .clicked()
                {
                    self.selected_phrase = i;
                    // Changing phrase clears event selection
                    self.selected_event_step = None;
                }
            }
        });

        ui.separator();

        // Phrase editor
        if self.selected_phrase < song.phrase_bank.len() {
            ui.label(format!("Editing Phrase {:02X}", self.selected_phrase));
            ui.separator();
            let phrase = &mut song.phrase_bank[self.selected_phrase];
            let text_height = ui.text_style_height(&egui::TextStyle::Monospace);

            TableBuilder::new(ui)
                .striped(true)
                .column(Column::auto()) // step
                .column(Column::auto()) // note
                .column(Column::auto()) // vol
                .column(Column::auto()) // instr/effect (read-only)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.label("Step");
                    });
                    header.col(|ui| {
                        ui.label("Note");
                    });
                    header.col(|ui| {
                        ui.label("Vol");
                    });
                    header.col(|ui| {
                        ui.label("Instr/FX");
                    });
                })
                .body(|mut body| {
                    for (step, event) in phrase.events.iter_mut().enumerate() {
                        body.row(text_height + 4.0, |mut row| {
                            // Step (click to select this event)
                            row.col(|ui| {
                                let is_selected = self.selected_event_step == Some(step);
                                if ui
                                    .selectable_label(is_selected, format!("{:02X}", step))
                                    .clicked()
                                {
                                    if is_selected {
                                        // Toggle off selection if clicking again
                                        self.selected_event_step = None;
                                    } else {
                                        self.selected_event_step = Some(step);
                                    }
                                }
                            });
                            // Note editable
                            row.col(|ui| {
                                let key = (self.selected_phrase, step);
                                let buf = ui_state.phrases_note.entry(key).or_insert_with(|| {
                                    if event.note == 0 {
                                        String::new()
                                    } else {
                                        format!("{:X}", event.note)
                                    }
                                });
                                let _response =
                                    hex_u8_editor(ui, buf, &mut event.note, text_height * 2.4);
                            });
                            // Volume editable
                            row.col(|ui| {
                                let key = (self.selected_phrase, step);
                                let buf = ui_state.phrases_vol.entry(key).or_insert_with(|| {
                                    if event.volume == 0 {
                                        String::new()
                                    } else {
                                        format!("{:X}", event.volume)
                                    }
                                });
                                let _response =
                                    hex_u8_editor(ui, buf, &mut event.volume, text_height * 2.4);
                            });
                            // Instrument assignment + effect display
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    // Instrument selector
                                    let current = event.instrument_id;
                                    let selected_text = if current == 0 {
                                        "--".to_string()
                                    } else if let Some(inst) = song
                                        .instrument_bank
                                        .iter()
                                        .find(|i| i.id == current as usize)
                                    {
                                        format!("{:02X} {}", current, inst.name)
                                    } else {
                                        format!("{:02X}", current)
                                    };
                                    egui::ComboBox::from_id_salt((
                                        "evt_inst",
                                        self.selected_phrase,
                                        step,
                                    ))
                                    .width(140.0)
                                    .selected_text(selected_text)
                                    .show_ui(ui, |ui| {
                                        if ui.selectable_label(current == 0, "--").clicked() {
                                            event.instrument_id = 0;
                                        }
                                        for inst in &song.instrument_bank {
                                            let id_u8 = (inst.id as u8).min(u8::MAX);
                                            let label = format!("{:02X} {}", id_u8, inst.name);
                                            let is_sel = current == id_u8;
                                            if ui.selectable_label(is_sel, label).clicked() {
                                                event.instrument_id = id_u8;
                                            }
                                        }
                                    });
                                    ui.label(format!("FX: {:?}", event.effect));
                                });
                            });
                        });
                    }
                });
        }
    }
}
