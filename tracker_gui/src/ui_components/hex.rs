use eframe::egui;

/// Edit a u8 value as two-digit hexadecimal text.
/// Displays "--" as the empty/sentinel value (mapped to 0 on change),
/// returns the egui::Response of the text edit widget.
pub fn hex_u8_editor(ui: &mut egui::Ui, v: &mut u8, width_px: f32) -> egui::Response {
    let mut text = if *v == 0 {
        "--".to_string()
    } else {
        format!("{:02X}", *v)
    };

    let response = ui.add(
        egui::TextEdit::singleline(&mut text)
            .desired_width(width_px)
            .font(egui::TextStyle::Monospace),
    );

    if response.changed() {
        if text == "--" || text.is_empty() {
            *v = 0;
        } else if let Ok(parsed) = u8::from_str_radix(&text, 16) {
            *v = parsed;
        }
    }

    response
}

/// Edit a usize value using a two-digit hexadecimal text editor with a sentinel value.
/// If the current value equals `sentinel`, shows "--". When edited to "--" or empty,
/// sets the value back to `sentinel`. Otherwise parses a hex u8 (00..FF) into the usize.
/// Returns the egui::Response of the text edit widget.
pub fn hex_usize_with_sentinel_editor(
    ui: &mut egui::Ui,
    v: &mut usize,
    sentinel: usize,
    width_px: f32,
) -> egui::Response {
    let mut text = if *v == sentinel {
        "--".to_string()
    } else if *v <= 0xFF {
        format!("{:02X}", *v as u8)
    } else {
        // Fallback for larger numbers; not expected in current UI but avoids surprises
        format!("{:X}", *v)
    };

    let response = ui.add(
        egui::TextEdit::singleline(&mut text)
            .desired_width(width_px)
            .font(egui::TextStyle::Monospace),
    );

    if response.changed() {
        if text == "--" || text.is_empty() {
            *v = sentinel;
        } else if let Ok(parsed) = u8::from_str_radix(&text, 16) {
            *v = parsed as usize;
        }
    }

    response
}

