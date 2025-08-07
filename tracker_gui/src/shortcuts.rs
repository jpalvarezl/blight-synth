use eframe::egui;

pub struct ShortcutHandler;

pub enum ShortcutAction {
    TogglePlayback,
    LoadSong,
    SaveSong,
    QuitApplication,
    None,
}

impl ShortcutHandler {
    pub fn handle_shortcuts(ctx: &egui::Context) -> ShortcutAction {
        ctx.input(|i| {
            // Spacebar: Toggle playback
            if i.key_pressed(egui::Key::Space) {
                return ShortcutAction::TogglePlayback;
            }
            
            // Cmd+O: Open/Load Song
            if i.modifiers.command && i.key_pressed(egui::Key::O) {
                return ShortcutAction::LoadSong;
            }
            
            // Cmd+S: Save Song (defaults to JSON format)
            if i.modifiers.command && i.key_pressed(egui::Key::S) {
                return ShortcutAction::SaveSong;
            }
            
            // Cmd+Q: Quit application
            if i.modifiers.command && i.key_pressed(egui::Key::Q) {
                return ShortcutAction::QuitApplication;
            }
            
            ShortcutAction::None
        })
    }
    
    pub fn show_shortcuts_window(ctx: &egui::Context, show_shortcuts_window: &mut bool) {
        if *show_shortcuts_window {
            egui::Window::new("Keyboard Shortcuts")
                .open(show_shortcuts_window)
                .default_width(400.0)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Blight Tracker - Keyboard Shortcuts");
                    ui.separator();
                    ui.add_space(10.0);
                    
                    egui::Grid::new("shortcuts_grid")
                        .num_columns(2)
                        .spacing([20.0, 8.0])
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new("Action").strong());
                            ui.label(egui::RichText::new("Shortcut").strong());
                            ui.end_row();
                            
                            ui.label("Toggle Playback");
                            ui.label(egui::RichText::new("Space").code());
                            ui.end_row();
                            
                            ui.label("Open/Load Song");
                            ui.label(egui::RichText::new("⌘+O").code());
                            ui.end_row();
                            
                            ui.label("Save Song (JSON)");
                            ui.label(egui::RichText::new("⌘+S").code());
                            ui.end_row();
                            
                            ui.label("Quit Application");
                            ui.label(egui::RichText::new("⌘+Q").code());
                            ui.end_row();
                        });
                });
        }
    }
}
