use eframe::egui;
use sequencer::cli::FileFormat;
use sequencer::models::Song;

use crate::audio::AudioManager;
use crate::file_ops::FileOperations;
use crate::instrument_manager::InstrumentManagerWindow;
use crate::menu::{MenuActions, MenuRenderer, ShortcutAction, ShortcutHandler};
use crate::tabs::{
    CurrentTab, arrangement::ArrangementTab, chains::ChainsTab, phrases::PhrasesTab,
};
use crate::theme::ThemeManager;
use crate::ui_components::{EffectType, SidePanel, SidePanelAction, SongInfoEditor, TabSelector};
use crate::ui_state::UiState;
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
    pub side_panel: SidePanel,
    pub ui_state: UiState,
    pub instrument_window: InstrumentManagerWindow,
}

impl TrackerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
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

    fn handle_menu_actions(&mut self, actions: MenuActions, ctx: &egui::Context) {
        if actions.new_song {
            let new_song = FileOperations::new_song();
            self.load_song_data(new_song, ctx);
        }

        if actions.load_song {
            if let Some(song) = FileOperations::load_song() {
                self.load_song_data(song, ctx);
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

        if actions.show_shortcuts {
            self.show_shortcuts_window = true;
        }

        if actions.toggle_theme {
            self.theme_manager.toggle_theme(ctx);
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

    fn handle_effect_selection(
        &mut self,
        effect: EffectType,
        _current_track: usize,
        event_selection: Option<(usize, usize)>,
    ) {
        // Update the sequencer model so the UI reflects the effect immediately
        if let (EffectType::Reverb, Some((phrase_idx, step_idx))) = (effect, event_selection) {
            if phrase_idx < self.song.phrase_bank.len()
                && step_idx < self.song.phrase_bank[phrase_idx].events.len()
            {
                let event = &mut self.song.phrase_bank[phrase_idx].events[step_idx];
                event.effect = sequencer::models::EffectType::SetReverb;
                if event.effect_param == 0 {
                    event.effect_param = 1;
                }
            }
        }

        if let Some(audio) = &mut self.audio_manager.audio {
            match (effect, event_selection) {
                (EffectType::Reverb, Some((phrase_idx, step_idx))) => {
                    if phrase_idx < self.song.phrase_bank.len()
                        && step_idx < self.song.phrase_bank[phrase_idx].events.len()
                    {
                        let inst_id =
                            self.song.phrase_bank[phrase_idx].events[step_idx].instrument_id as u32;
                        if inst_id != 0 {
                            // Monophonic instrument: add a single reverb effect instance
                            let r = audio.get_effect_factory().create_mono_reverb();
                            audio.send_command(
                                audio_backend::SequencerCmd::AddEffectToInstrument {
                                    instrument_id: audio_backend::id::InstrumentId::from(inst_id),
                                    effect: r,
                                }
                                .into(),
                            );
                            log::info!("Added Reverb to instrument {}", inst_id);
                        } else {
                            log::warn!(
                                "Cannot add effect: selected event has no instrument (inherit)"
                            );
                        }
                    }
                }
                _ => {}
            }
        } else {
            // Attempt to initialize audio silently for a smoother UX
            self.audio_manager.init_audio(&self.song);
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
            side_panel: SidePanel::default(),
            ui_state: UiState::default(),
            instrument_window: InstrumentManagerWindow::default(),
        }
    }
}

impl eframe::App for TrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_shortcuts(ctx);

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            let actions = MenuRenderer::show_menu_bar(
                ui,
                ctx,
                self.audio_manager.is_playing,
                &self.theme_manager,
            );
            self.handle_menu_actions(actions, ctx);
        });

        let current_track = self.arrangement_tab.current_track;

        let event_selection = self
            .phrases_tab
            .selected_event_step
            .map(|step| (self.phrases_tab.selected_phrase, step));
        if let Some(action) =
            self.side_panel
                .show(ctx, current_track, event_selection, &mut self.song)
        {
            match action {
                SidePanelAction::AddEffect(effect) => {
                    self.handle_effect_selection(effect, current_track, event_selection);
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Blight Tracker - M8 Style Interface");
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
    }
}
