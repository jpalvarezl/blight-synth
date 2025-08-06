use eframe::egui;
use sequencer::models::{Song, Phrase, EffectType};

#[derive(Default)]
pub struct PhrasesTab {
    pub selected_phrase: usize,
}

impl PhrasesTab {
    pub fn reset(&mut self) {
        self.selected_phrase = 0;
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
                    if self.selected_phrase >= song.phrase_bank.len() && !song.phrase_bank.is_empty() {
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
                if ui.selectable_label(i == self.selected_phrase, format!("{:02X}", i)).clicked() {
                    self.selected_phrase = i;
                }
            }
        });
        
        ui.separator();
        
        // Phrase editor
        if self.selected_phrase < song.phrase_bank.len() {
            ui.label(format!("Editing Phrase {:02X}", self.selected_phrase));
            ui.separator();
            
            // Headers for the phrase editor
            ui.horizontal(|ui| {
                ui.label("Step");
                ui.separator();
                ui.label("Note");
                ui.separator();
                ui.label("Vol");
            });
            
            ui.separator();
            
            let phrase = &mut song.phrase_bank[self.selected_phrase];
            
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for (step, event) in phrase.events.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{:02X}:", step));
                            ui.separator();
                            
                            // Note field
                            let mut note_text = if event.note == 0 {
                                "--".to_string()
                            } else {
                                format!("{:02X}", event.note)
                            };
                            
                            let note_response = ui.add(
                                egui::TextEdit::singleline(&mut note_text)
                                    .desired_width(40.0)
                                    .font(egui::TextStyle::Monospace)
                            );
                            
                            if note_response.changed() {
                                if note_text == "--" || note_text.is_empty() {
                                    event.note = 0;
                                } else if let Ok(parsed) = u8::from_str_radix(&note_text, 16) {
                                    event.note = parsed;
                                }
                            }
                            
                            ui.separator();
                            
                            // Volume field
                            let mut vol_text = if event.volume == 0 {
                                "--".to_string()
                            } else {
                                format!("{:02X}", event.volume)
                            };
                            
                            let vol_response = ui.add(
                                egui::TextEdit::singleline(&mut vol_text)
                                    .desired_width(40.0)
                                    .font(egui::TextStyle::Monospace)
                            );
                            
                            if vol_response.changed() {
                                if vol_text == "--" || vol_text.is_empty() {
                                    event.volume = 0;
                                } else if let Ok(parsed) = u8::from_str_radix(&vol_text, 16) {
                                    event.volume = parsed;
                                }
                            }
                            
                            // Keep effect as default (no effect)
                            event.effect = EffectType::Arpeggio;
                            event.effect_param = 0;
                        });
                    }
                });
        }
    }
}
