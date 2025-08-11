use crate::tabs::CurrentTab;
use eframe::egui;
use sequencer::models::Song;

pub mod side_panel;
pub use side_panel::{AvailableInstrument, EffectType, SidePanel, SidePanelAction};

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
            if ui.text_edit_singleline(song_name).changed() {
                song.name = song_name.clone();
            }

            ui.separator();

            ui.label("BPM:");
            if ui.text_edit_singleline(bpm).changed() {
                if let Ok(bpm_val) = bpm.parse::<u16>() {
                    song.initial_bpm = bpm_val;
                }
            }

            ui.separator();

            ui.label("Speed:");
            if ui.text_edit_singleline(speed).changed() {
                if let Ok(speed_val) = speed.parse::<u16>() {
                    song.initial_speed = speed_val;
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
