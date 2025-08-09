use crate::theme::ThemeManager;
use eframe::egui;

pub struct MenuActions {
    pub new_song: bool,
    pub load_song: bool,
    pub save_json: bool,
    pub save_binary: bool,
    pub quit: bool,
    pub toggle_playback: bool,
    pub init_audio: bool,
    pub show_shortcuts: bool,
    pub toggle_theme: bool,
}

impl Default for MenuActions {
    fn default() -> Self {
        Self {
            new_song: false,
            load_song: false,
            save_json: false,
            save_binary: false,
            quit: false,
            toggle_playback: false,
            init_audio: false,
            show_shortcuts: false,
            toggle_theme: false,
        }
    }
}

pub struct MenuRenderer;

impl MenuRenderer {
    pub fn show_menu_bar(
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        is_playing: bool,
        theme_manager: &ThemeManager,
    ) -> MenuActions {
        let mut actions = MenuActions::default();

        egui::MenuBar::new().ui(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                if ui.button("New Song").clicked() {
                    actions.new_song = true;
                    ui.close();
                }

                ui.separator();

                if ui.button("Load Song").clicked() {
                    actions.load_song = true;
                    ui.close();
                }

                ui.separator();

                if ui.button("Export as JSON").clicked() {
                    actions.save_json = true;
                    ui.close();
                }

                if ui.button("Export as Binary").clicked() {
                    actions.save_binary = true;
                    ui.close();
                }

                ui.separator();

                if ui.button("Quit").clicked() {
                    actions.quit = true;
                    ui.close();
                }
            });

            // Playback menu
            ui.menu_button("Playback", |ui| {
                let play_text = if is_playing { "⏸ Stop" } else { "▶ Play" };

                if ui.button(play_text).clicked() {
                    actions.toggle_playback = true;
                    ui.close();
                }

                ui.separator();

                if ui.button("🔄 Initialize Audio").clicked() {
                    actions.init_audio = true;
                    ui.close();
                }
            });

            // Help menu
            ui.menu_button("Help", |ui| {
                if ui.button("Shortcuts").clicked() {
                    actions.show_shortcuts = true;
                    ui.close();
                }
            });

            // Theme toggle button on the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .button(theme_manager.theme_button_emoji())
                    .on_hover_text(theme_manager.theme_button_tooltip())
                    .clicked()
                {
                    actions.toggle_theme = true;
                }
            });
        });

        if actions.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        actions
    }
}
