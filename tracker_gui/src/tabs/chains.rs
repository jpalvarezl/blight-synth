use eframe::egui;
use egui_extras::{TableBuilder, Column};
use sequencer::models::{Song, Chain, EMPTY_PHRASE_SLOT};

#[derive(Default)]
pub struct ChainsTab {
    pub selected_chain: usize,
}

impl ChainsTab {
    pub fn reset(&mut self) {
        self.selected_chain = 0;
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, song: &mut Song) {
        ui.horizontal(|ui| {
            ui.label(format!("Chains: {} total", song.chain_bank.len()));
            if ui.button("Add Chain").clicked() {
                song.chain_bank.push(Chain::default());
            }
            if ui.button("Remove Chain").clicked() && !song.chain_bank.is_empty() {
                if self.selected_chain < song.chain_bank.len() {
                    song.chain_bank.remove(self.selected_chain);
                    if self.selected_chain >= song.chain_bank.len() && !song.chain_bank.is_empty() {
                        self.selected_chain = song.chain_bank.len() - 1;
                    }
                }
            }
        });
        
        ui.separator();
        
        if song.chain_bank.is_empty() {
            ui.label("No chains available. Click 'Add Chain' to create one.");
            return;
        }
        
        // Chain selector
        ui.horizontal(|ui| {
            ui.label("Chain:");
            for (i, _) in song.chain_bank.iter().enumerate() {
                if ui.selectable_label(i == self.selected_chain, format!("{:02X}", i)).clicked() {
                    self.selected_chain = i;
                }
            }
        });
        
        ui.separator();
        
        // Chain editor
        if self.selected_chain < song.chain_bank.len() {
            ui.label(format!("Editing Chain {:02X}", self.selected_chain));
            ui.separator();
            
            let chain = &mut song.chain_bank[self.selected_chain];
            let text_height = ui.text_style_height(&egui::TextStyle::Monospace);
            
            TableBuilder::new(ui)
                .striped(true)
                .column(Column::auto())
                .column(Column::auto())
                .header(20.0, |mut header| {
                    header.col(|ui| { ui.label("Step"); });
                    header.col(|ui| { ui.label("Phrase"); });
                })
                .body(|mut body| {
                    for (step, phrase_idx) in chain.phrase_indices.iter_mut().enumerate() {
                        body.row(text_height + 4.0, |mut row| {
                            row.col(|ui| {
                                ui.label(format!("{:02X}", step));
                            });
                            row.col(|ui| {
                                let mut phrase_text = if *phrase_idx == EMPTY_PHRASE_SLOT {
                                    "--".to_string()
                                } else {
                                    format!("{:02X}", *phrase_idx)
                                };
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut phrase_text)
                                        .desired_width(text_height * 2.4)
                                        .font(egui::TextStyle::Monospace)
                                );
                                if response.changed() {
                                    if phrase_text == "--" || phrase_text.is_empty() {
                                        *phrase_idx = EMPTY_PHRASE_SLOT;
                                    } else if let Ok(parsed) = u8::from_str_radix(&phrase_text, 16) {
                                        *phrase_idx = parsed as usize;
                                    }
                                }
                            });
                        });
                    }
                });
        }
    }
}
