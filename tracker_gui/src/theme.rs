use eframe::egui;
use egui::{Color32, CornerRadius, FontFamily, FontId, TextStyle};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone)]
pub struct ThemePalette {
    pub window_fill: Color32,
    pub panel_fill: Color32,
    pub accent_strong: Color32,
    pub accent_soft: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub outline: Color32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeSource {
    BuiltIn,
    Custom,
}

#[derive(Clone)]
pub struct ThemeProfile {
    pub id: String,
    pub display_name: String,
    pub button_icon: String,
    pub palette: ThemePalette,
    pub dark_mode: bool,
    pub source: ThemeSource,
}

pub struct ThemeDescriptor<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub icon: &'a str,
}

const DEFAULT_CUSTOM_ICON: &str = "🎨";

#[derive(Debug)]
pub enum ThemeConfigError {
    Json(serde_json::Error),
    InvalidColor(String),
}

impl fmt::Display for ThemeConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThemeConfigError::Json(err) => write!(f, "{err}"),
            ThemeConfigError::InvalidColor(value) => {
                write!(f, "invalid color value '{value}', expected #RRGGBB hex")
            }
        }
    }
}

impl std::error::Error for ThemeConfigError {}

#[derive(Serialize, Deserialize)]
struct SerializableTheme {
    #[serde(default)]
    id: Option<String>,
    name: String,
    #[serde(default)]
    icon: Option<String>,
    #[serde(default)]
    dark_mode: Option<bool>,
    palette: SerializablePalette,
}

#[derive(Serialize, Deserialize)]
struct SerializablePalette {
    window_fill: String,
    panel_fill: String,
    accent_strong: String,
    accent_soft: String,
    text_primary: String,
    text_secondary: String,
    outline: String,
}

impl ThemeProfile {
    fn m8_night_shift() -> Self {
        Self {
            id: "m8-night".to_string(),
            display_name: "M8 Night".to_string(),
            button_icon: "⬛".to_string(),
            dark_mode: true,
            palette: ThemePalette {
                window_fill: Color32::from_rgb(6, 10, 18),
                panel_fill: Color32::from_rgb(12, 18, 32),
                accent_strong: Color32::from_rgb(73, 255, 210),
                accent_soft: Color32::from_rgb(52, 122, 150),
                text_primary: Color32::from_rgb(224, 250, 244),
                text_secondary: Color32::from_rgb(118, 146, 160),
                outline: Color32::from_rgb(36, 52, 70),
            },
            source: ThemeSource::BuiltIn,
        }
    }

    fn m8_skyline() -> Self {
        Self {
            id: "m8-skyline".to_string(),
            display_name: "M8 Skyline".to_string(),
            button_icon: "🟦".to_string(),
            dark_mode: false,
            palette: ThemePalette {
                window_fill: Color32::from_rgb(238, 244, 250),
                panel_fill: Color32::from_rgb(218, 228, 240),
                accent_strong: Color32::from_rgb(44, 120, 180),
                accent_soft: Color32::from_rgb(118, 164, 210),
                text_primary: Color32::from_rgb(12, 24, 38),
                text_secondary: Color32::from_rgb(68, 86, 110),
                outline: Color32::from_rgb(180, 196, 214),
            },
            source: ThemeSource::BuiltIn,
        }
    }

    fn from_serializable(theme: SerializableTheme, source: ThemeSource) -> Result<Self, ThemeConfigError> {
        let id = theme
            .id
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| slugify_id(&theme.name));

        Ok(Self {
            id,
            display_name: theme.name,
            button_icon: theme.icon.unwrap_or_else(|| DEFAULT_CUSTOM_ICON.to_string()),
            palette: ThemePalette::from_serializable(theme.palette)?,
            dark_mode: theme.dark_mode.unwrap_or(true),
            source,
        })
    }
}

impl ThemePalette {
    fn from_serializable(palette: SerializablePalette) -> Result<Self, ThemeConfigError> {
        Ok(Self {
            window_fill: color_from_hex(&palette.window_fill)?,
            panel_fill: color_from_hex(&palette.panel_fill)?,
            accent_strong: color_from_hex(&palette.accent_strong)?,
            accent_soft: color_from_hex(&palette.accent_soft)?,
            text_primary: color_from_hex(&palette.text_primary)?,
            text_secondary: color_from_hex(&palette.text_secondary)?,
            outline: color_from_hex(&palette.outline)?,
        })
    }

    fn to_serializable(&self) -> SerializablePalette {
        SerializablePalette {
            window_fill: color_to_hex(self.window_fill),
            panel_fill: color_to_hex(self.panel_fill),
            accent_strong: color_to_hex(self.accent_strong),
            accent_soft: color_to_hex(self.accent_soft),
            text_primary: color_to_hex(self.text_primary),
            text_secondary: color_to_hex(self.text_secondary),
            outline: color_to_hex(self.outline),
        }
    }
}

impl From<&ThemeProfile> for SerializableTheme {
    fn from(value: &ThemeProfile) -> Self {
        SerializableTheme {
            id: Some(value.id.clone()),
            name: value.display_name.clone(),
            icon: Some(value.button_icon.clone()),
            dark_mode: Some(value.dark_mode),
            palette: value.palette.to_serializable(),
        }
    }
}

pub struct ThemeManager {
    profiles: Vec<ThemeProfile>,
    active_index: usize,
}

impl ThemeManager {
    fn new_with_defaults() -> Self {
        let mut manager = Self {
            profiles: Vec::new(),
            active_index: 0,
        };

        manager.register_theme(ThemeProfile::m8_night_shift());
        manager.register_theme(ThemeProfile::m8_skyline());
        manager
    }

    pub fn apply_theme(&self, ctx: &egui::Context) {
        let theme = self.current_theme();
        ctx.set_fonts(configure_fonts());

        let mut style = (*ctx.style()).clone();
        configure_text_sizes(&mut style);
        configure_spacing(&mut style);
        configure_visuals(&mut style, theme);

        ctx.set_style(style);
    }

    pub fn cycle_theme(&mut self, ctx: &egui::Context) {
        if self.profiles.is_empty() {
            return;
        }

        self.active_index = (self.active_index + 1) % self.profiles.len();
        self.apply_theme(ctx);
    }

    pub fn register_theme(&mut self, profile: ThemeProfile) {
        if let Some(existing) = self
            .profiles
            .iter_mut()
            .find(|candidate| candidate.id == profile.id)
        {
            *existing = profile;
        } else {
            self.profiles.push(profile);
        }
    }

    pub fn set_active_theme(&mut self, id: &str, ctx: &egui::Context) -> bool {
        if let Some((index, _)) = self
            .profiles
            .iter()
            .enumerate()
            .find(|(_, profile)| profile.id == id)
        {
            self.active_index = index;
            self.apply_theme(ctx);
            true
        } else {
            false
        }
    }

    pub fn active_theme_name(&self) -> &str {
        &self.current_theme().display_name
    }

    pub fn active_theme_id(&self) -> &str {
        &self.current_theme().id
    }

    pub fn available_themes(&self) -> impl Iterator<Item = ThemeDescriptor<'_>> {
        self.profiles.iter().map(|profile| ThemeDescriptor {
            id: profile.id.as_str(),
            name: profile.display_name.as_str(),
            icon: profile.button_icon.as_str(),
        })
    }

    pub fn import_theme_from_str(
        &mut self,
        raw: &str,
    ) -> Result<ThemeProfile, ThemeConfigError> {
        let serializable: SerializableTheme =
            serde_json::from_str(raw).map_err(ThemeConfigError::Json)?;
        let profile = ThemeProfile::from_serializable(serializable, ThemeSource::Custom)?;
        let cloned = profile.clone();
        self.register_theme(profile);
        Ok(cloned)
    }

    pub fn restore_custom_themes(&mut self, raw: &str) -> Result<usize, ThemeConfigError> {
        if raw.trim().is_empty() {
            return Ok(0);
        }

        let entries: Vec<SerializableTheme> =
            serde_json::from_str(raw).map_err(ThemeConfigError::Json)?;
        let mut count = 0;
        for entry in entries {
            let profile = ThemeProfile::from_serializable(entry, ThemeSource::Custom)?;
            self.register_theme(profile);
            count += 1;
        }
        Ok(count)
    }

    pub fn export_custom_themes_json(&self) -> Result<Option<String>, ThemeConfigError> {
        let custom: Vec<_> = self
            .profiles
            .iter()
            .filter(|profile| matches!(profile.source, ThemeSource::Custom))
            .collect();

        if custom.is_empty() {
            return Ok(None);
        }

        let serializable: Vec<SerializableTheme> =
            custom.into_iter().map(SerializableTheme::from).collect();
        let json = serde_json::to_string_pretty(&serializable).map_err(ThemeConfigError::Json)?;
        Ok(Some(json))
    }

    fn current_theme(&self) -> &ThemeProfile {
        self
            .profiles
            .get(self.active_index)
            .or_else(|| self.profiles.first())
            .expect("ThemeManager must contain at least one theme")
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new_with_defaults()
    }
}

fn configure_fonts() -> egui::FontDefinitions {
    egui::FontDefinitions::default()
}

fn configure_text_sizes(style: &mut egui::Style) {
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::new(24.0, FontFamily::Monospace));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(16.0, FontFamily::Monospace));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::new(15.0, FontFamily::Monospace));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::new(12.0, FontFamily::Monospace));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::new(16.0, FontFamily::Monospace));
}

fn configure_spacing(style: &mut egui::Style) {
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.button_padding = egui::vec2(10.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(10);
    style.spacing.interact_size = egui::vec2(24.0, 20.0);
}

fn configure_visuals(style: &mut egui::Style, profile: &ThemeProfile) {
    let mut visuals = if profile.dark_mode {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    let palette = &profile.palette;

    visuals.override_text_color = Some(palette.text_primary);
    visuals.window_fill = palette.window_fill;
    visuals.panel_fill = palette.panel_fill;
    visuals.faint_bg_color = palette.window_fill;
    visuals.extreme_bg_color = palette.panel_fill;
    visuals.hyperlink_color = palette.accent_strong;
    visuals.selection.bg_fill = palette.accent_strong;
    visuals.selection.stroke.color = palette.panel_fill;
    visuals.widgets.noninteractive.bg_fill = palette.panel_fill;
    visuals.widgets.noninteractive.fg_stroke.color = palette.text_secondary;
    visuals.widgets.inactive.bg_fill = palette.panel_fill;
    visuals.widgets.inactive.fg_stroke.color = palette.text_primary;
    visuals.widgets.hovered.bg_fill = palette.accent_soft;
    visuals.widgets.hovered.fg_stroke.color = palette.window_fill;
    visuals.widgets.active.bg_fill = palette.accent_strong;
    visuals.widgets.active.fg_stroke.color = palette.window_fill;
    visuals.window_stroke.color = palette.outline;
    visuals.widgets.noninteractive.bg_stroke.color = palette.outline;
    visuals.widgets.inactive.bg_stroke.color = palette.outline;
    visuals.widgets.hovered.bg_stroke.color = palette.accent_strong;
    visuals.widgets.active.bg_stroke.color = palette.accent_strong;
    visuals.window_corner_radius = CornerRadius::same(3);
    visuals.menu_corner_radius = CornerRadius::same(3);

    style.visuals = visuals;
}

fn color_from_hex(input: &str) -> Result<Color32, ThemeConfigError> {
    let trimmed = input.trim().trim_start_matches('#');
    if trimmed.len() != 6 {
        return Err(ThemeConfigError::InvalidColor(input.to_string()));
    }

    let r = u8::from_str_radix(&trimmed[0..2], 16)
        .map_err(|_| ThemeConfigError::InvalidColor(input.to_string()))?;
    let g = u8::from_str_radix(&trimmed[2..4], 16)
        .map_err(|_| ThemeConfigError::InvalidColor(input.to_string()))?;
    let b = u8::from_str_radix(&trimmed[4..6], 16)
        .map_err(|_| ThemeConfigError::InvalidColor(input.to_string()))?;
    Ok(Color32::from_rgb(r, g, b))
}

fn color_to_hex(color: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

fn slugify_id(input: &str) -> String {
    let mut slug: String = input
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '-'
            }
        })
        .collect();

    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }

    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        "custom-theme".to_string()
    } else {
        slug.to_string()
    }
}
