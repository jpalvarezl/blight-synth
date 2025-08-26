use eframe::egui;
use egui_extras::{Column, TableBuilder};
use sequencer::models::{MAX_TRACKS, Song};
use crate::ui_components::hex_usize_with_sentinel_editor;

pub struct ArrangementTab {
    pub current_row: usize,
    pub current_track: usize,
    scroll_to_current: bool,
}

impl Default for ArrangementTab {
    fn default() -> Self {
        Self {
            current_row: 0,
            current_track: 0,
            scroll_to_current: false,
        }
    }
}

impl ArrangementTab {
    pub fn reset(&mut self) {
        self.current_row = 0;
        self.current_track = 0;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, song: &mut Song) {
        ui.label(egui::RichText::new("Arrangement").heading());
        ui.add_space(4.0);

        // Control buttons
        ui.horizontal(|ui| {
            if ui.button("Add Row").clicked() {
                song.arrangement.push(Default::default());
                self.current_row = song.arrangement.len() - 1;
                self.scroll_to_current = true;
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
        ui.add_space(4.0);

        let text_height = ui.text_style_height(&egui::TextStyle::Monospace);
        let row_height = text_height + 4.0;
        let visible_rows = 16.0;
        let scroll_height = row_height * visible_rows + 24.0;

        // Build table: first column Row, then one per track
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .min_scrolled_height(scroll_height)
            .max_scroll_height(scroll_height)
            .column(Column::auto()) // Row number
            .resizable(false);
        for _ in 0..MAX_TRACKS {
            table = table.column(Column::auto());
        }

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label("Row");
                });
                for track in 0..MAX_TRACKS {
                    header.col(|ui| {
                        ui.label(format!("Track {}", track + 1));
                    });
                }
            })
            .body(|mut body| {
                for (row_idx, song_row) in song.arrangement.iter_mut().enumerate() {
                    let is_current_row = row_idx == self.current_row;
                    body.row(text_height + 4.0, |mut row| {
                        // Row number cell
                        row.col(|ui| {
                            let label =
                                ui.selectable_label(is_current_row, format!("{:02X}", row_idx));
                            if is_current_row && self.scroll_to_current {
                                label.scroll_to_me(Some(egui::Align::Center));
                            }
                            if label.clicked() {
                                self.current_row = row_idx;
                            }
                        });

                        // Track columns
                        for track in 0..MAX_TRACKS {
                            row.col(|ui| {
                                let chain_idx = &mut song_row.chain_indices[track];
                                let response = hex_usize_with_sentinel_editor(
                                    ui,
                                    chain_idx,
                                    usize::MAX,
                                    text_height * 2.4,
                                );

                                if response.clicked() {
                                    self.current_track = track;
                                    self.current_row = row_idx;
                                }

                                // Column highlight for current track
                                if track == self.current_track {
                                    ui.painter().rect_filled(
                                        response.rect,
                                        0.0,
                                        egui::Color32::from_rgba_unmultiplied(60, 60, 90, 80),
                                    );
                                }
                            });
                        }
                    });
                }
            });

        self.scroll_to_current = false;

        // Status bar anchored at bottom
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Current: Row {:02X}, Track {}",
                    self.current_row,
                    self.current_track + 1
                ));
                ui.separator();
                ui.label(format!("Arrangement Length: {}", song.arrangement.len()));
            });
        });
    }
}
