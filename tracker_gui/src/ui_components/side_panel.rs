use audio_backend::{InstrumentDefinition, id::InstrumentId};
use eframe::egui;
use std::collections::HashMap;

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

    pub fn get_instrument_id(&self) -> InstrumentId {
        match self {
            AvailableInstrument::Oscillator => InstrumentId::from(1u32),
            AvailableInstrument::SamplePlayer => InstrumentId::from(0u32),
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
pub struct EffectItem {
    pub effect: EffectType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectChain {
    pub id: String,
    pub name: String,
    pub items: Vec<EffectItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InstrumentStub {
    pub id: String,
    pub name: String,
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
        event_selection: Option<(usize, usize)>, // (phrase_idx, step_idx)
        instrument_bank: &mut Vec<InstrumentStub>,
        effect_chain_bank: &mut Vec<EffectChain>,
        instrument_assignments: &mut HashMap<(usize, usize), String>,
        chain_assignments: &mut HashMap<(usize, usize), String>,
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

                    // Bottom half: Inspector
                    ui.group(|ui| {
                        ui.set_height(half_height);
                        ui.vertical(|ui| {
                            ui.heading("Inspector");
                            ui.separator();

                            match event_selection {
                                Some((phrase_idx, step_idx)) => {
                                    ui.label(format!(
                                        "Selected Event: Phrase {:02X}, Step {:02X}",
                                        phrase_idx, step_idx
                                    ));
                                    ui.add_space(6.0);

                                    // Instrument assignment dropdown
                                    ui.label("Instrument:");
                                    let current_instr_id = instrument_assignments
                                        .get(&(phrase_idx, step_idx))
                                        .cloned();
                                    let mut selected_instr_index: usize = current_instr_id
                                        .as_ref()
                                        .and_then(|id| {
                                            instrument_bank.iter().position(|i| &i.id == id)
                                        })
                                        .unwrap_or(0);
                                    egui::ComboBox::from_id_source("event_instrument_combo")
                                        .width(180.0)
                                        .selected_text(
                                            instrument_bank
                                                .get(selected_instr_index)
                                                .map(|i| i.name.as_str())
                                                .unwrap_or("<none>"),
                                        )
                                        .show_ui(ui, |ui| {
                                            for (idx, inst) in instrument_bank.iter().enumerate() {
                                                ui.selectable_value(
                                                    &mut selected_instr_index,
                                                    idx,
                                                    &inst.name,
                                                );
                                            }
                                        });
                                    if let Some(inst) = instrument_bank.get(selected_instr_index) {
                                        instrument_assignments
                                            .insert((phrase_idx, step_idx), inst.id.clone());
                                        // TODO: Apply instrument assignment to backend event when available
                                    }

                                    ui.add_space(8.0);

                                    // Effect Chain assignment dropdown
                                    ui.label("Effect Chain:");
                                    let current_chain_id =
                                        chain_assignments.get(&(phrase_idx, step_idx)).cloned();
                                    let mut selected_chain_index: usize = current_chain_id
                                        .as_ref()
                                        .and_then(|id| {
                                            effect_chain_bank.iter().position(|c| &c.id == id)
                                        })
                                        .unwrap_or(0);
                                    egui::ComboBox::from_id_source("event_chain_combo")
                                        .width(180.0)
                                        .selected_text(
                                            effect_chain_bank
                                                .get(selected_chain_index)
                                                .map(|c| c.name.as_str())
                                                .unwrap_or("<none>"),
                                        )
                                        .show_ui(ui, |ui| {
                                            for (idx, chain) in effect_chain_bank.iter().enumerate()
                                            {
                                                ui.selectable_value(
                                                    &mut selected_chain_index,
                                                    idx,
                                                    &chain.name,
                                                );
                                            }
                                        });
                                    if let Some(chain) = effect_chain_bank.get(selected_chain_index)
                                    {
                                        chain_assignments
                                            .insert((phrase_idx, step_idx), chain.id.clone());
                                        // TODO: Apply effect chain assignment to backend event when available
                                    }

                                    ui.add_space(8.0);
                                    ui.separator();

                                    // Read-only chain preview
                                    ui.label("Chain Preview:");
                                    egui::ScrollArea::vertical()
                                        .auto_shrink([false, true])
                                        .show(ui, |ui| {
                                            if let Some(chain_id) =
                                                chain_assignments.get(&(phrase_idx, step_idx))
                                            {
                                                if let Some(chain) = effect_chain_bank
                                                    .iter()
                                                    .find(|c| &c.id == chain_id)
                                                {
                                                    for item in &chain.items {
                                                        ui.label(format!(
                                                            "• {}",
                                                            item.effect.name()
                                                        ));
                                                    }
                                                } else {
                                                    ui.label("<no chain>");
                                                }
                                            } else {
                                                ui.label("<no chain>");
                                            }
                                        });
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
