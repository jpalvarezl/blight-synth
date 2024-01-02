#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;

mod event_handlers;
mod ui_components;

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
            ui_components::init_ui(ui, self);
            event_handlers::handle_input(ctx, self);
        });
    }
}
