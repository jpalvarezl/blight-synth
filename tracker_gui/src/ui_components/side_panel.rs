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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectType {
    Reverb,
    Delay,
    Distortion,
    BitCrush,
    Filter,
    Gain,
}

impl EffectType {
    pub fn name(&self) -> &'static str {
        match self {
            EffectType::Reverb => "Reverb",
            EffectType::Delay => "Delay",
            EffectType::Distortion => "Distortion",
            EffectType::BitCrush => "Bit crush",
            EffectType::Filter => "Filter",
            EffectType::Gain => "Gain",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            EffectType::Reverb,
            EffectType::Delay,
            EffectType::Distortion,
            EffectType::BitCrush,
            EffectType::Filter,
            EffectType::Gain,
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentStub {
    pub id: String,
    pub name: String,
}

pub enum SidePanelAction {
    AssignInstrumentToSelectedEvent(AvailableInstrument),
    AddEffect(EffectType),
}

enum InstrumentAction {
    None,
    ApplyToEvent(AvailableInstrument),
}

pub struct SidePanel {
    pub is_expanded: bool,
    pub panel_width: f32,
    pub selected_instrument: AvailableInstrument,
    pub selected_effect: EffectType,
}
impl Default for SidePanel {
    fn default() -> Self {
        Self {
            is_expanded: true,
            panel_width: 250.0,
            selected_instrument: AvailableInstrument::Oscillator,
            selected_effect: EffectType::Reverb,
        }
    }
}

impl SidePanel {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        current_track: usize,
        event_selection: Option<(usize, usize)>, // (phrase_idx, step_idx)
    ) -> Option<SidePanelAction> {
        let mut action: Option<SidePanelAction> = None;

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
                                .id_salt(egui::Id::new("instruments_scroll"))
                                .auto_shrink([false, true])
                                .show(ui, |ui| match self.show_instrument_selector(ui) {
                                    InstrumentAction::ApplyToEvent(instr) => {
                                        action = Some(
                                            SidePanelAction::AssignInstrumentToSelectedEvent(instr),
                                        );
                                    }
                                    InstrumentAction::None => {}
                                });
                        });
                    });

                    ui.add_space(5.0);

                    // Bottom half: Effects
                    ui.group(|ui| {
                        ui.set_height(half_height);
                        ui.vertical(|ui| {
                            ui.heading("Effects");
                            ui.separator();

                            // Single-effect selection per event/instrument
                            match event_selection {
                                Some((_phrase_idx, _step_idx)) => {
                                    ui.label("Effect:");
                                    let effects = EffectType::all();
                                    // Map current selection to index
                                    let mut selected_idx = effects
                                        .iter()
                                        .position(|e| *e == self.selected_effect)
                                        .unwrap_or(0);
                                    egui::ComboBox::from_id_salt("event_effect_combo")
                                        .width(180.0)
                                        .selected_text(
                                            effects
                                                .get(selected_idx)
                                                .map(|e| e.name())
                                                .unwrap_or("<none>"),
                                        )
                                        .show_ui(ui, |ui| {
                                            for (idx, eff) in effects.iter().enumerate() {
                                                let enabled = matches!(eff, EffectType::Reverb);
                                                ui.add_enabled_ui(enabled, |ui| {
                                                    if ui
                                                        .selectable_label(
                                                            selected_idx == idx,
                                                            eff.name(),
                                                        )
                                                        .clicked()
                                                    {
                                                        selected_idx = idx;
                                                    }
                                                });
                                            }
                                        });
                                    // Update stored effect selection
                                    if let Some(sel) = effects.get(selected_idx) {
                                        self.selected_effect = *sel;
                                    }

                                    ui.add_space(8.0);
                                    // Apply button (only Reverb supported for now)
                                    let can_apply =
                                        matches!(self.selected_effect, EffectType::Reverb);
                                    ui.add_enabled_ui(can_apply, |ui| {
                                        if ui.button("Apply to Selected Event").clicked() {
                                            action = Some(SidePanelAction::AddEffect(
                                                self.selected_effect,
                                            ));
                                        }
                                    });

                                    ui.add_space(8.0);
                                    ui.separator();
                                    ui.label("Only Reverb is available for now.");
                                }
                                None => {
                                    ui.label(
                                        "No event selected. Select an event in Phrases to inspect.",
                                    );
                                }
                            }
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

        action
    }

    fn show_instrument_selector(&mut self, ui: &mut egui::Ui) -> InstrumentAction {
        let mut apply_to_event: Option<AvailableInstrument> = None;

        for instrument in AvailableInstrument::all() {
            let is_available = instrument.to_instrument_definition().is_some();
            let is_selected = instrument == self.selected_instrument;

            ui.horizontal(|ui| {
                // Show selection radio button
                if ui.radio(is_selected, "").clicked() && is_available {
                    self.selected_instrument = instrument.clone();
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

        // Per-event apply button
        let can_apply = self
            .selected_instrument
            .to_instrument_definition()
            .is_some();
        ui.add_enabled_ui(can_apply, |ui| {
            if ui.button("Apply to Selected Event").clicked() {
                apply_to_event = Some(self.selected_instrument.clone());
            }
        });

        if !can_apply {
            ui.label(
                egui::RichText::new("Selected instrument not available")
                    .small()
                    .color(egui::Color32::GRAY),
            );
        }

        if let Some(instr) = apply_to_event {
            return InstrumentAction::ApplyToEvent(instr);
        }
        InstrumentAction::None
    }
}
