use crate::theme::ThemeManager;
use eframe::egui;

pub struct MenuActions {
    pub new_song: bool,
    pub load_song: bool,
    pub save_json: bool,
    pub save_binary: bool,
    pub quit: bool,
    pub toggle_playback: bool,
    pub toggle_looping: bool,
    pub show_instrument_manager: bool,
    pub show_shortcuts: bool,
    pub toggle_theme: bool,
    pub select_theme: Option<String>,
    pub import_theme: bool,
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
            toggle_looping: false,
            show_instrument_manager: false,
            show_shortcuts: false,
            toggle_theme: false,
            select_theme: None,
            import_theme: false,
        }
    }
}

pub struct MenuRenderer;

impl MenuRenderer {
    pub fn show_menu_bar(
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        is_playing: bool,
        loop_enabled: bool,
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

                // Loop playback toggle
                let mut loop_state = loop_enabled;
                if ui.checkbox(&mut loop_state, "Loop playback").clicked() {
                    actions.toggle_looping = true;
                    ui.close();
                }
            });

            // Instruments menu
            ui.menu_button("Instruments", |ui| {
                if ui.button("Manage Instruments…").clicked() {
                    actions.show_instrument_manager = true;
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

            ui.menu_button("Theme", |ui| {
                for descriptor in theme_manager.available_themes() {
                    let label = format!("{} {}", descriptor.icon, descriptor.name);
                    let is_active = theme_manager.active_theme_id() == descriptor.id;
                    if ui.radio(is_active, label).clicked() {
                        actions.select_theme = Some(descriptor.id.to_string());
                        ui.close();
                    }
                }
                ui.separator();
                if ui.button("Cycle Theme").clicked() {
                    actions.toggle_theme = true;
                    ui.close();
                }
                if ui.button("Import Theme…").clicked() {
                    actions.import_theme = true;
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
