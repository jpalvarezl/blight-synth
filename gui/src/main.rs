#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use audio_backend::{get_default_output_device_name, start_audio_thread};
use eframe::egui;
use event_handlers::{keyboard::PianoKeyboard, InputStateHandler};
use view_models::oscillator::OscillatorViewModel;

mod event_handlers;
mod ui_components;
mod view_models;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions::default();
    let initial_content = init_content(get_default_output_device_name()?);

    let oscillator = initial_content.oscillator_viewmodel.get_oscillator();
    start_audio_thread(oscillator.clone());

    eframe::run_native(
        "Keyboard events",
        options,
        Box::new(|_cc| Box::new(initial_content)),
    )
    .map_err(|e| anyhow::anyhow!("eframe error: {}", e))
}

fn init_content(default_output_device: String) -> Content {
    Content {
        default_output_device,
        text: String::new(),
        input_handler: PianoKeyboard::initialize(), // This should use Box dyn somehow swap keyboards
        oscillator_viewmodel: OscillatorViewModel::new(),
    }
}

struct Content {
    default_output_device: String,
    text: String,
    input_handler: PianoKeyboard,
    oscillator_viewmodel: OscillatorViewModel,
}

impl eframe::App for Content {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui_components::init_ui(ui, self);
            InputStateHandler::handle_input(
                &mut self.input_handler,
                ctx,
                &self.oscillator_viewmodel,
            );

            self.text = self.input_handler.pressed_keys_as_string();

            println!("Oscillator: {:#?}", self.oscillator_viewmodel.get_oscillator());
        });
    }
}
