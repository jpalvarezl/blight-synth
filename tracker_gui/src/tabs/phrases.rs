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

    pub fn show(&mut self, ui: &mut egui::Ui, song: &mut Song) {
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
                                let mut note_text = if event.note == 0 {
                                    "--".to_string()
                                } else {
                                    format!("{:02X}", event.note)
                                };
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut note_text)
                                        .desired_width(text_height * 2.4)
                                        .font(egui::TextStyle::Monospace),
                                );
                                if response.changed() {
                                    if note_text == "--" || note_text.is_empty() {
                                        event.note = 0;
                                    } else if let Ok(parsed) = u8::from_str_radix(&note_text, 16) {
                                        event.note = parsed;
                                    }
                                }
                            });
                            // Volume editable
                            row.col(|ui| {
                                let mut vol_text = if event.volume == 0 {
                                    "--".to_string()
                                } else {
                                    format!("{:02X}", event.volume)
                                };
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut vol_text)
                                        .desired_width(text_height * 2.4)
                                        .font(egui::TextStyle::Monospace),
                                );
                                if response.changed() {
                                    if vol_text == "--" || vol_text.is_empty() {
                                        event.volume = 0;
                                    } else if let Ok(parsed) = u8::from_str_radix(&vol_text, 16) {
                                        event.volume = parsed;
                                    }
                                }
                            });
                            // Read-only instrument/effect column
                            row.col(|ui| {
                                ui.label(format!(
                                    "{:02X} / {:?}",
                                    event.instrument_id,
                                    event.effect
                                ));
                            });
                        });
                    }
                });
        }
    }
}
