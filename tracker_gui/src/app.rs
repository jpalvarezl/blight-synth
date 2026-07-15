use eframe::egui;
use log::error;
use rfd::FileDialog;
use sequencer::cli::FileFormat;
use sequencer::models::Song;
use std::fs;
use std::time::{Duration, Instant};

use crate::audio::AudioManager;
use crate::file_ops::FileOperations;
use crate::instrument_manager::InstrumentManagerWindow;
use crate::menu::{MenuActions, MenuRenderer, ShortcutAction, ShortcutHandler};
use crate::tabs::{
    CurrentTab, arrangement::ArrangementTab, chains::ChainsTab, phrases::PhrasesTab,
};
use crate::theme::ThemeManager;
use crate::ui_components::{SongInfoEditor, TabSelector};
use crate::ui_state::UiState;

struct ThemeFeedback {
    message: String,
    is_error: bool,
    expires_at: Instant,
}

const STORAGE_THEME_ID: &str = "tracker.theme.active";
const STORAGE_CUSTOM_THEMES: &str = "tracker.theme.custom";
/// Duration (in seconds) to display theme-import feedback without being disruptive.
const THEME_FEEDBACK_SECS: u64 = 6;
pub struct TrackerApp {
    pub song: Song,
    pub song_name: String,
    pub bpm: String,
    pub speed: String,
    pub current_tab: CurrentTab,

    pub arrangement_tab: ArrangementTab,
    pub chains_tab: ChainsTab,
    pub phrases_tab: PhrasesTab,

    pub audio_manager: AudioManager,
    pub theme_manager: ThemeManager,

    pub show_shortcuts_window: bool,
    pub ui_state: UiState,
    pub instrument_window: InstrumentManagerWindow,
    theme_feedback: Option<ThemeFeedback>,
}

impl TrackerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        if let Some(storage) = cc.storage {
            app.restore_theme_preferences(storage, &cc.egui_ctx);
        }
        app.theme_manager.apply_theme(&cc.egui_ctx);
        // Auto-initialize audio engine on startup (idempotent)
        app.audio_manager.init_audio(&app.song);
        app
    }

    fn reset_tab_states(&mut self) {
        self.arrangement_tab.reset();
        self.chains_tab.reset();
        self.phrases_tab.reset();
    }

    fn load_song_data(&mut self, song: Song, ctx: &egui::Context) {
        self.song = song;
        self.song_name = self.song.name.clone();
        self.bpm = self.song.initial_bpm.to_string();
        self.speed = self.song.initial_speed.to_string();
        self.reset_tab_states();

        // Clear UI input buffers so editors reflect the new song
        self.ui_state = UiState::default();
        // Also clear egui memory to reset any lingering widget state
        ctx.memory_mut(|mem| mem.data.clear());

        // Rehydrate audio engine from the newly loaded song
        if self.audio_manager.audio.is_some() {
            self.audio_manager.reset_with_song(&self.song);
        }
    }

    fn restore_theme_preferences(&mut self, storage: &dyn eframe::Storage, ctx: &egui::Context) {
        if let Some(json) = storage.get_string(STORAGE_CUSTOM_THEMES)
            && let Err(err) = self.theme_manager.restore_custom_themes(&json)
        {
            error!("Failed to restore custom themes: {err}");
        }

        if let Some(theme_id) = storage.get_string(STORAGE_THEME_ID) {
            let trimmed = theme_id.trim();
            if !trimmed.is_empty() && !self.theme_manager.set_active_theme(trimmed, ctx) {
                error!("Stored theme '{trimmed}' was not found. Keeping current theme");
            }
        }
    }

    fn import_theme_via_dialog(&mut self, ctx: &egui::Context) {
        if let Some(path) = FileDialog::new()
            .set_title("Import Theme")
            .add_filter("Theme JSON", &["json"])
            .pick_file()
        {
            match fs::read_to_string(&path) {
                Ok(contents) => match self.theme_manager.import_theme_from_str(&contents) {
                    Ok(profile) => {
                        self.theme_manager.set_active_theme(&profile.id, ctx);
                        let display_name = profile.display_name;
                        self.set_theme_feedback(format!("Imported theme '{display_name}'"), false);
                    }
                    Err(err) => {
                        self.set_theme_feedback(format!("Theme import failed: {err}"), true)
                    }
                },
                Err(err) => {
                    self.set_theme_feedback(
                        format!("Could not read {}: {err}", path.display()),
                        true,
                    );
                }
            }
        }
    }

    fn set_theme_feedback(&mut self, message: impl Into<String>, is_error: bool) {
        self.theme_feedback = Some(ThemeFeedback {
            message: message.into(),
            is_error,
            expires_at: Instant::now() + Duration::from_secs(THEME_FEEDBACK_SECS),
        });
    }

    fn prune_theme_feedback(&mut self) {
        if let Some(feedback) = &self.theme_feedback
            && feedback.expires_at <= Instant::now()
        {
            self.theme_feedback = None;
        }
    }

    fn handle_menu_actions(&mut self, actions: MenuActions, ctx: &egui::Context) {
        if actions.new_song {
            let new_song = FileOperations::new_song();
            self.load_song_data(new_song, ctx);
        }

        if actions.load_song
            && let Some(song) = FileOperations::load_song()
        {
            self.load_song_data(song, ctx);
        }

        if actions.save_json {
            FileOperations::save_song(&self.song, FileFormat::Json);
        }

        if actions.save_binary {
            FileOperations::save_song(&self.song, FileFormat::Binary);
        }

        if actions.toggle_playback {
            self.audio_manager.toggle_playback(&self.song);
        }

        if actions.toggle_looping {
            self.audio_manager.toggle_looping();
        }

        if actions.show_shortcuts {
            self.show_shortcuts_window = true;
        }

        if actions.toggle_theme {
            self.theme_manager.cycle_theme(ctx);
        }

        if let Some(theme_id) = actions.select_theme {
            self.theme_manager.set_active_theme(&theme_id, ctx);
        }

        if actions.import_theme {
            self.import_theme_via_dialog(ctx);
        }

        if actions.show_instrument_manager {
            self.instrument_window.open = true;
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let action = ShortcutHandler::handle_shortcuts(ctx);
        match action {
            ShortcutAction::TogglePlayback => self.audio_manager.toggle_playback(&self.song),
            ShortcutAction::NextTab => self.current_tab = self.current_tab.next(),
            ShortcutAction::PreviousTab => self.current_tab = self.current_tab.previous(),
            ShortcutAction::LoadSong => {
                if let Some(song) = FileOperations::load_song() {
                    self.load_song_data(song, ctx);
                }
            }
            ShortcutAction::SaveSong => FileOperations::save_song(&self.song, FileFormat::Json),
            ShortcutAction::QuitApplication => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            ShortcutAction::None => {}
        }
    }
}

impl Default for TrackerApp {
    fn default() -> Self {
        Self {
            song: Song::new("New Song"),
            song_name: "New Song".to_string(),
            bpm: "120".to_string(),
            speed: "6".to_string(),
            current_tab: CurrentTab::Arrangement,
            arrangement_tab: ArrangementTab::default(),
            chains_tab: ChainsTab::default(),
            phrases_tab: PhrasesTab::default(),
            audio_manager: AudioManager::default(),
            theme_manager: ThemeManager::default(),
            show_shortcuts_window: false,
            ui_state: UiState::default(),
            instrument_window: InstrumentManagerWindow::default(),
            theme_feedback: None,
        }
    }
}

impl eframe::App for TrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.prune_theme_feedback();
        self.handle_shortcuts(ctx);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            let actions = MenuRenderer::show_menu_bar(
                ui,
                ctx,
                self.audio_manager.is_playing,
                self.audio_manager.loop_enabled,
                &self.theme_manager,
            );
            self.handle_menu_actions(actions, ctx);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!(
                "Blight Tracker — {}",
                self.theme_manager.active_theme_name()
            ));
            ui.separator();

            SongInfoEditor::show(
                ui,
                &mut self.song,
                &mut self.song_name,
                &mut self.bpm,
                &mut self.speed,
            );
            ui.separator();

            TabSelector::show(ui, &mut self.current_tab);
            ui.separator();

            match self.current_tab {
                CurrentTab::Arrangement => {
                    self.arrangement_tab
                        .show(ui, &mut self.song, &mut self.ui_state)
                }
                CurrentTab::Chains => self.chains_tab.show(ui, &mut self.song, &mut self.ui_state),
                CurrentTab::Phrases => {
                    self.phrases_tab
                        .show(ui, &mut self.song, &mut self.ui_state)
                }
            }
        });

        ShortcutHandler::show_shortcuts_window(ctx, &mut self.show_shortcuts_window);

        // Instruments manager window
        self.instrument_window
            .show(ctx, &mut self.song, &mut self.audio_manager);

        if let Some(feedback) = self.theme_feedback.as_ref() {
            let color = if feedback.is_error {
                egui::Color32::from_rgb(255, 140, 140)
            } else {
                egui::Color32::from_rgb(150, 255, 210)
            };
            let text = feedback.message.clone();
            egui::TopBottomPanel::bottom("theme_feedback_panel").show(ctx, |ui| {
                ui.add_space(4.0);
                ui.colored_label(color, text);
            });
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string(
            STORAGE_THEME_ID,
            self.theme_manager.active_theme_id().to_string(),
        );

        match self.theme_manager.export_custom_themes_json() {
            Ok(Some(json)) => storage.set_string(STORAGE_CUSTOM_THEMES, json),
            Ok(None) => storage.set_string(STORAGE_CUSTOM_THEMES, String::new()),
            Err(err) => error!("Failed to persist custom themes: {err}"),
        }
    }
}
