use eframe::egui;
use sequencer::cli::FileFormat;
use sequencer::models::Song;
use std::collections::HashMap;

use crate::audio::AudioManager;
use crate::file_ops::FileOperations;
use crate::menu::{MenuActions, MenuRenderer, ShortcutAction, ShortcutHandler};
use crate::tabs::{
    CurrentTab, arrangement::ArrangementTab, chains::ChainsTab, phrases::PhrasesTab,
};
use crate::theme::ThemeManager;
use crate::ui_components::{
    AvailableInstrument, EffectChain, EffectItem, EffectType, SidePanel, SongInfoEditor,
    TabSelector,
};

pub struct TrackerApp {
    // Song data
    pub song: Song,
    pub song_name: String,
    pub bpm: String,
    pub speed: String,
    pub current_tab: CurrentTab,

    // Tab handlers
    pub arrangement_tab: ArrangementTab,
    pub chains_tab: ChainsTab,
    pub phrases_tab: PhrasesTab,

    // System managers
    pub audio_manager: AudioManager,
    pub theme_manager: ThemeManager,

    // UI state
    pub show_shortcuts_window: bool,
    pub side_panel: SidePanel,

    // GUI-side banks and assignments (mock only)
    pub effect_chain_bank: Vec<EffectChain>,
    pub chain_assignments: HashMap<(usize, usize), String>, // (phrase_idx, step_idx) -> chain_id
}

impl TrackerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let app = Self::default();
        app.theme_manager.apply_theme(&cc.egui_ctx);
        app
    }

    fn reset_tab_states(&mut self) {
        self.arrangement_tab.reset();
        self.chains_tab.reset();
        self.phrases_tab.reset();
    }

    fn load_song_data(&mut self, song: Song) {
        self.song = song;
        self.song_name = self.song.name.clone();
        self.bpm = self.song.initial_bpm.to_string();
        self.speed = self.song.initial_speed.to_string();
        self.reset_tab_states();
    }

    fn handle_menu_actions(&mut self, actions: MenuActions, ctx: &egui::Context) {
        if actions.new_song {
            let new_song = FileOperations::new_song();
            self.load_song_data(new_song);
        }

        if actions.load_song {
            if let Some(song) = FileOperations::load_song() {
                self.load_song_data(song);
            }
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

        if actions.init_audio {
            self.audio_manager.init_audio(&self.song);
        }

        if actions.show_shortcuts {
            self.show_shortcuts_window = true;
        }

        if actions.toggle_theme {
            self.theme_manager.toggle_theme(ctx);
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let action = ShortcutHandler::handle_shortcuts(ctx);

        match action {
            ShortcutAction::TogglePlayback => {
                self.audio_manager.toggle_playback(&self.song);
            }
            ShortcutAction::NextTab => {
                self.current_tab = self.current_tab.next();
            }
            ShortcutAction::PreviousTab => {
                self.current_tab = self.current_tab.previous();
            }
            ShortcutAction::LoadSong => {
                if let Some(song) = FileOperations::load_song() {
                    self.load_song_data(song);
                }
            }
            ShortcutAction::SaveSong => {
                FileOperations::save_song(&self.song, FileFormat::Json);
            }
            ShortcutAction::QuitApplication => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            ShortcutAction::None => {}
        }
    }

    fn handle_instrument_selection(&mut self, instrument: AvailableInstrument) {
        if let Some(instrument_def) = instrument.to_instrument_definition() {
            match self
                .audio_manager
                .set_track_instrument(instrument.get_instrument_id(), instrument_def)
            {
                Ok(_) => {
                    log::info!("Set instrument: {:?}", instrument.name());
                }
                Err(error) => {
                    log::error!("Failed to set instrument: {}", error);
                    // TODO: Consider showing this error in the UI as well
                }
            }
        }
    }
}

impl Default for TrackerApp {
    fn default() -> Self {
        // Mock banks
        let effect_chain_bank = vec![EffectChain {
            id: "default".to_string(),
            name: "Default Chain".to_string(),
            items: EffectType::all()
                .into_iter()
                .map(|e| EffectItem { effect: e })
                .collect(),
        }];

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
            side_panel: SidePanel::default(),
            effect_chain_bank,
            chain_assignments: HashMap::new(),
        }
    }
}

impl eframe::App for TrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts first
        self.handle_shortcuts(ctx);

        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            let actions = MenuRenderer::show_menu_bar(
                ui,
                ctx,
                self.audio_manager.is_playing,
                &self.theme_manager,
            );
            self.handle_menu_actions(actions, ctx);
        });

        // Get current track from arrangement tab
        let current_track = self.arrangement_tab.current_track;

        // Side panel (must be shown before CentralPanel)
        let event_selection = self
            .phrases_tab
            .selected_event_step
            .map(|step| (self.phrases_tab.selected_phrase, step));
        if let Some(selected_instrument) = self.side_panel.show(
            ctx,
            current_track,
            event_selection,
            &mut self.effect_chain_bank,
            &mut self.chain_assignments,
        ) {
            self.handle_instrument_selection(selected_instrument);
        }

        // Main content area
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Blight Tracker - M8 Style Interface");
            ui.separator();

            // Song info editor
            SongInfoEditor::show(
                ui,
                &mut self.song,
                &mut self.song_name,
                &mut self.bpm,
                &mut self.speed,
            );
            ui.separator();

            // Tab selector
            TabSelector::show(ui, &mut self.current_tab);
            ui.separator();

            // Tab content
            match self.current_tab {
                CurrentTab::Arrangement => self.arrangement_tab.show(ui, &mut self.song),
                CurrentTab::Chains => self.chains_tab.show(ui, &mut self.song),
                CurrentTab::Phrases => self.phrases_tab.show(ui, &mut self.song),
            }
        });

        // Show shortcuts window if requested
        ShortcutHandler::show_shortcuts_window(ctx, &mut self.show_shortcuts_window);
    }
}
