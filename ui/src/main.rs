#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use egui::*;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    
    let options = eframe::NativeOptions::default();
    let initial_content = init_content(core::get_default_output_device_name()?);
    
    eframe::run_native(
        "Keyboard events",
        options,
        Box::new(|_cc| Box::new(initial_content)),
    ).map_err(|e| anyhow::anyhow!("eframe error: {}", e))
}

fn init_content(default_output_device: String) -> Content {
    Content {
        default_output_device,
        text: String::new(),
    }
}

#[derive(Default)]
struct Content {
    default_output_device: String,
    text: String,
}

impl eframe::App for Content {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.default_output_device);
            ui.heading("Press/Hold/Release example. Press A to test.");
    
            if ui.button("Clear").clicked() {
                self.text.clear();
            }
            ScrollArea::vertical()
                .auto_shrink(false)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.label(&self.text);
                });

            if ctx.input(|i| i.key_pressed(Key::A)) {
                self.text.push_str("\nPressed");
            }
            if ctx.input(|i| i.key_down(Key::A)) {
                self.text.push_str("\nHeld");
                ui.ctx().request_repaint(); // make sure we note the holding.
            }
            if ctx.input(|i| i.key_released(Key::A)) {
                self.text.push_str("\nReleased");
            }
        });
    }
}
