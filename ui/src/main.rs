#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use event_handlers::{keyboard::PianoKeyboard, InputStateHandler};
use core::{get_default_output_device_name, play_the_thing};

mod event_handlers;
mod ui_components;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    
    let options = eframe::NativeOptions::default();
    let initial_content = init_content(get_default_output_device_name()?);

    play_the_thing()?;
    
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
        input_handler: PianoKeyboard::initialize(), // This should use Box dyn somehow swap keyboards
    }
}

struct Content {
    default_output_device: String,
    text: String,
    input_handler: PianoKeyboard
}

impl eframe::App for Content {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui_components::init_ui(ui, self);
            InputStateHandler::handle_input(&mut self.input_handler, ctx);

            self.text = self.input_handler.pressed_keys_as_string();
        });
    }
}
