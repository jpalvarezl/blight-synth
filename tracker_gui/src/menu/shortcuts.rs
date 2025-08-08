use eframe::egui::{self, Key, KeyboardShortcut, Modifiers};

pub struct ShortcutHandler;

pub enum ShortcutAction {
    TogglePlayback,
    LoadSong,
    SaveSong,
    QuitApplication,
    NextTab,
    PreviousTab,
    None,
}

/// Define all keyboard shortcuts as constants
pub mod shortcuts {
    use super::*;

    pub const TOGGLE_PLAYBACK: KeyboardShortcut =
        KeyboardShortcut::new(Modifiers::NONE, Key::Space);
    pub const NEXT_TAB: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::Tab);
    pub const PREVIOUS_TAB: KeyboardShortcut =
        KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::Tab);
    pub const LOAD_SONG: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
    pub const SAVE_SONG: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::S);
    pub const QUIT_APP: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Q);
}

impl ShortcutHandler {
    pub fn handle_shortcuts(ctx: &egui::Context) -> ShortcutAction {
        // Handle shortcuts using proper egui KeyboardShortcut consumption
        // This automatically handles focus and prevents conflicts

        if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::TOGGLE_PLAYBACK)) {
            return ShortcutAction::TogglePlayback;
        }

        // Check tab cycling shortcuts (most specific first)
        if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::PREVIOUS_TAB)) {
            return ShortcutAction::PreviousTab;
        }

        if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::NEXT_TAB)) {
            return ShortcutAction::NextTab;
        }

        if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::LOAD_SONG)) {
            return ShortcutAction::LoadSong;
        }

        if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::SAVE_SONG)) {
            return ShortcutAction::SaveSong;
        }

        if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::QUIT_APP)) {
            return ShortcutAction::QuitApplication;
        }

        ShortcutAction::None
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

                            ui.label("Next Tab");
                            ui.label(egui::RichText::new("Ctrl+Tab").code());
                            ui.end_row();

                            ui.label("Previous Tab");
                            ui.label(egui::RichText::new("Shift+Ctrl+Tab").code());
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
