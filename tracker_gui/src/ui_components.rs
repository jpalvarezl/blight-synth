use crate::tabs::CurrentTab;
use eframe::egui;
use sequencer::models::Song;

pub mod side_panel;
pub use side_panel::{EffectType, SidePanel, SidePanelAction};

pub mod hex;
pub use hex::{dec_u8_editor, hex_u8_editor, hex_usize_with_sentinel_editor};

pub struct SongInfoEditor;

impl SongInfoEditor {
    pub fn show(
        ui: &mut egui::Ui,
        song: &mut Song,
        song_name: &mut String,
        bpm: &mut String,
        speed: &mut String,
    ) {
        ui.horizontal(|ui| {
            ui.label("Song:");
            let resp_name = ui.text_edit_singleline(song_name);
            if resp_name.lost_focus() && resp_name.changed() {
                song.name = song_name.clone();
            }

            ui.separator();

            ui.label("BPM:");
            let resp_bpm = ui.text_edit_singleline(bpm);
            if resp_bpm.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(bpm_val) = bpm.parse::<u16>() {
                    song.initial_bpm = bpm_val;
                } else {
                    // Revert to last valid value
                    *bpm = song.initial_bpm.to_string();
                }
            }

            ui.separator();

            ui.label("Speed:");
            let resp_speed = ui.text_edit_singleline(speed);
            if resp_speed.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if let Ok(speed_val) = speed.parse::<u16>() {
                    song.initial_speed = speed_val;
                } else {
                    // Revert to last valid value
                    *speed = song.initial_speed.to_string();
                }
            }
        });
    }
}

pub struct TabSelector;

impl TabSelector {
    pub fn show(ui: &mut egui::Ui, current_tab: &mut CurrentTab) {
        ui.horizontal(|ui| {
            ui.selectable_value(current_tab, CurrentTab::Arrangement, "Arrangement");
            ui.selectable_value(current_tab, CurrentTab::Chains, "Chains");
            ui.selectable_value(current_tab, CurrentTab::Phrases, "Phrases");
        });
    }
}
