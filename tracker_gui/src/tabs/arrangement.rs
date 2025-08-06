use eframe::egui;
use sequencer::models::{Song, MAX_TRACKS};

#[derive(Default)]
pub struct ArrangementTab {
    pub current_row: usize,
    pub current_track: usize,
}

impl ArrangementTab {
    pub fn reset(&mut self) {
        self.current_row = 0;
        self.current_track = 0;
    }
    
    pub fn show(&mut self, ui: &mut egui::Ui, song: &mut Song) {
        // Track headers
        ui.horizontal(|ui| {
            ui.label("Row");
            ui.separator();
            for track in 0..MAX_TRACKS {
                ui.label(format!("Track {}", track + 1));
                if track < MAX_TRACKS - 1 {
                    ui.separator();
                }
            }
        });

        ui.separator();

        // Arrangement grid (song rows)
        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for (row_idx, song_row) in song.arrangement.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        // Row number with highlighting for current row
                        let row_label = ui.selectable_label(
                            row_idx == self.current_row,
                            format!("{:02X}", row_idx)
                        );
                        if row_label.clicked() {
                            self.current_row = row_idx;
                        }
                        
                        ui.separator();
                        
                        // Chain indices for each track
                        for track in 0..MAX_TRACKS {
                            let chain_idx = &mut song_row.chain_indices[track];
                            let mut chain_text = if *chain_idx == usize::MAX {
                                "--".to_string()
                            } else {
                                format!("{:02X}", *chain_idx)
                            };
                            
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut chain_text)
                                    .desired_width(30.0)
                                    .font(egui::TextStyle::Monospace)
                            );
                            
                            if response.changed() {
                                if chain_text == "--" || chain_text.is_empty() {
                                    *chain_idx = usize::MAX;
                                } else if let Ok(parsed) = u8::from_str_radix(&chain_text, 16) {
                                    *chain_idx = parsed as usize;
                                }
                            }
                            
                            if response.clicked() {
                                self.current_track = track;
                                self.current_row = row_idx;
                            }
                            
                            if track < MAX_TRACKS - 1 {
                                ui.separator();
                            }
                        }
                    });
                }
            });

        ui.separator();

        // Control buttons
        ui.horizontal(|ui| {
            if ui.button("Add Row").clicked() {
                song.arrangement.push(Default::default());
            }
            
            if ui.button("Remove Row").clicked() && !song.arrangement.is_empty() {
                if self.current_row < song.arrangement.len() {
                    song.arrangement.remove(self.current_row);
                    if self.current_row >= song.arrangement.len() && !song.arrangement.is_empty() {
                        self.current_row = song.arrangement.len() - 1;
                    }
                }
            }
        });

        // Status bar
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!("Current: Row {:02X}, Track {}", self.current_row, self.current_track + 1));
            ui.separator();
            ui.label(format!("Arrangement Length: {}", song.arrangement.len()));
        });
    }
}
