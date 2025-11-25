use eframe::egui;
use sequencer::models::AmpEnvelopeParams;

pub fn show_amp_envelope_editor(
    ui: &mut egui::Ui,
    params: &mut AmpEnvelopeParams,
    instrument_id: usize,
    ui_prefix: &'static str,
    mut on_change: impl FnMut(&AmpEnvelopeParams),
) {
    ui.push_id((ui_prefix, instrument_id as u32, "amp_env"), |ui| {
        egui::CollapsingHeader::new("Amplitude Envelope")
            .id_salt((ui_prefix, instrument_id as u32, "amp_env_hdr"))
            .show(ui, |ui| {
                let mut changed = false;
                let mut atk = params.attack;
                let mut dec = params.decay;
                let mut sus = params.sustain;
                let mut rel = params.release;

                ui.horizontal(|ui| {
                    ui.label("Attack");
                    changed |= ui
                        .add(egui::Slider::new(&mut atk, 0.0..=2.0).suffix(" s"))
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Decay");
                    changed |= ui
                        .add(egui::Slider::new(&mut dec, 0.0..=2.0).suffix(" s"))
                        .changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Sustain");
                    changed |= ui.add(egui::Slider::new(&mut sus, 0.0..=1.0)).changed();
                });
                ui.horizontal(|ui| {
                    ui.label("Release");
                    changed |= ui
                        .add(egui::Slider::new(&mut rel, 0.0..=5.0).suffix(" s"))
                        .changed();
                });

                if changed {
                    params.attack = atk;
                    params.decay = dec;
                    params.sustain = sus;
                    params.release = rel;
                    on_change(params);
                }
            });
    });
}
