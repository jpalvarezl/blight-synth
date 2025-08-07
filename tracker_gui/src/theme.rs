use eframe::egui;

pub struct ThemeManager {
    pub is_dark_mode: bool,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self {
            is_dark_mode: true,
        }
    }
}

impl ThemeManager {
    pub fn apply_theme(&self, ctx: &egui::Context) {
        // Use default fonts (monospace primary)
        ctx.set_fonts(egui::FontDefinitions::default());

        // Base visuals
        let mut style = (*ctx.style()).clone();
        style.visuals = if self.is_dark_mode {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        style.spacing.item_spacing = egui::vec2(4.0, 2.0);
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(40, 120, 200);
        ctx.set_style(style);
    }
    
    pub fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.is_dark_mode = !self.is_dark_mode;
        self.apply_theme(ctx);
    }
    
    pub fn theme_button_emoji(&self) -> &str {
        if self.is_dark_mode { "☀" } else { "🌙" }
    }
    
    pub fn theme_button_tooltip(&self) -> &str {
        if self.is_dark_mode { "Switch to light mode" } else { "Switch to dark mode" }
    }
}
