use audio_backend::InstrumentDefinition;
use eframe::egui;

#[derive(Debug, Clone, PartialEq)]
pub enum AvailableInstrument {
    Oscillator,
    SamplePlayer, // We'll disable this for now since it needs file loading
}

impl AvailableInstrument {
    pub fn name(&self) -> &'static str {
        match self {
            AvailableInstrument::Oscillator => "Oscillator",
            AvailableInstrument::SamplePlayer => "Sample Player",
        }
    }

    pub fn to_instrument_definition(&self) -> Option<InstrumentDefinition> {
        match self {
            AvailableInstrument::Oscillator => Some(InstrumentDefinition::Oscillator),
            AvailableInstrument::SamplePlayer => None, // TODO: Need sample loading
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Oscillator, Self::SamplePlayer]
    }
}

pub struct SidePanel {
    pub is_expanded: bool,
    pub panel_width: f32,
    pub selected_instrument: AvailableInstrument,
}

impl Default for SidePanel {
    fn default() -> Self {
        Self {
            is_expanded: true,
            panel_width: 250.0,
            selected_instrument: AvailableInstrument::Oscillator,
        }
    }
}

impl SidePanel {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        current_track: usize,
    ) -> Option<AvailableInstrument> {
        let mut selected_instrument = None;

        // Only show the panel if expanded
        if self.is_expanded {
            egui::SidePanel::right("side_panel")
                .default_width(self.panel_width)
                .min_width(200.0)
                .max_width(400.0)
                .resizable(true)
                .show(ctx, |ui| {
                    // Header with collapse button
                    ui.horizontal(|ui| {
                        ui.heading("Panel");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("◀").on_hover_text("Collapse panel").clicked() {
                                self.is_expanded = false;
                            }
                        });
                    });

                    ui.separator();

                    // Split the panel in half vertically
                    let available_height = ui.available_height();
                    let half_height = (available_height - 10.0) / 2.0; // 10.0 for spacing

                    // Top half: Instruments
                    ui.group(|ui| {
                        ui.set_height(half_height);
                        ui.vertical(|ui| {
                            ui.heading("Instruments");
                            ui.separator();

                            // Show current track info
                            ui.label(format!("Current Track: {}", current_track + 1));
                            ui.separator();

                            ui.label("Select instrument type:");
                            ui.add_space(5.0);

                            egui::ScrollArea::vertical()
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    selected_instrument = self.show_instrument_selector(ui);
                                });
                        });
                    });

                    ui.add_space(5.0);

                    // Bottom half: Effects (placeholder for now)
                    ui.group(|ui| {
                        ui.set_height(half_height);
                        ui.vertical(|ui| {
                            ui.heading("Effects");
                            ui.separator();
                            ui.label("Effects panel - coming soon!");
                        });
                    });
                });
        } else {
            // Show collapsed panel with expand button
            egui::SidePanel::right("side_panel_collapsed")
                .default_width(30.0)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        if ui.button("▶").on_hover_text("Expand panel").clicked() {
                            self.is_expanded = true;
                        }
                    });
                });
        }

        selected_instrument
    }

    fn show_instrument_selector(&mut self, ui: &mut egui::Ui) -> Option<AvailableInstrument> {
        let mut instrument_changed = None;

        for instrument in AvailableInstrument::all() {
            let is_available = instrument.to_instrument_definition().is_some();
            let is_selected = instrument == self.selected_instrument;

            ui.horizontal(|ui| {
                // Show selection radio button
                if ui.radio(is_selected, "").clicked() && is_available {
                    self.selected_instrument = instrument.clone();
                    instrument_changed = Some(instrument.clone());
                }

                // Instrument name with availability indication
                let text = if is_available {
                    egui::RichText::new(instrument.name())
                } else {
                    egui::RichText::new(instrument.name()).color(egui::Color32::GRAY)
                };
                ui.label(text);

                // Show "not available" indicator for disabled instruments
                if !is_available {
                    ui.label(
                        egui::RichText::new("(requires file)")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                }
            });

            ui.add_space(2.0);
        }

        ui.add_space(10.0);

        // "Apply to Track" button
        let can_apply = self
            .selected_instrument
            .to_instrument_definition()
            .is_some();
        ui.add_enabled_ui(can_apply, |ui| {
            if ui.button("Apply to Current Track").clicked() {
                instrument_changed = Some(self.selected_instrument.clone());
            }
        });

        if !can_apply {
            ui.label(
                egui::RichText::new("Selected instrument not available")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        }

        instrument_changed
    }
}
