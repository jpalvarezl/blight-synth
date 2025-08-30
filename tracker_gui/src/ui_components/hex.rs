use eframe::egui;

/// Edit a u8 as 2-digit hex with simple validation: allow only 0-9a-fA-F, max 2 chars.
/// Updates the numeric value as you type. The caller owns the text buffer.
pub fn hex_u8_editor(
    ui: &mut egui::Ui,
    buf: &mut String,
    v: &mut u8,
    width_px: f32,
) -> egui::Response {
    let mut text = buf.clone();
    let response = ui.add(
        egui::TextEdit::singleline(&mut text)
            .desired_width(width_px)
            .font(egui::TextStyle::Monospace),
    );

    if response.changed() {
        // Keep only hex chars and cap to 2
        let filtered: String = text
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .take(2)
            .collect();
        if filtered.is_empty() {
            *v = 0;
        } else if let Ok(parsed) = u8::from_str_radix(&filtered, 16) {
            *v = parsed;
        }
        if filtered != *buf {
            *buf = filtered;
        }
    }

    response
}

/// Edit a usize as 2-digit hex with simple validation and a sentinel for empty.
/// - Only 0-9a-fA-F allowed, max 2 chars.
/// - Empty string maps to `sentinel`. Caller owns the text buffer.
pub fn hex_usize_with_sentinel_editor(
    ui: &mut egui::Ui,
    buf: &mut String,
    v: &mut usize,
    sentinel: usize,
    width_px: f32,
) -> egui::Response {
    let mut text = buf.clone();
    let response = ui.add(
        egui::TextEdit::singleline(&mut text)
            .desired_width(width_px)
            .font(egui::TextStyle::Monospace),
    );

    if response.changed() {
        // Keep only hex chars and cap to 2
        let filtered: String = text
            .chars()
            .filter(|c| c.is_ascii_hexdigit())
            .take(2)
            .collect();
        if filtered.is_empty() {
            *v = sentinel;
        } else if let Ok(parsed) = u8::from_str_radix(&filtered, 16) {
            *v = parsed as usize;
        }
        if filtered != *buf {
            *buf = filtered;
        }
    }

    response
}
