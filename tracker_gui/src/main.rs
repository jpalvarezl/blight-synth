mod app;
mod audio;
mod audio_utils;
mod file_ops;
mod instrument_manager;
mod menu;
mod tabs;
mod theme;
mod ui_components;
mod ui_state;
use app::TrackerApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 750.0])
            .with_min_inner_size([800.0, 650.0])
            .with_title("Blight Tracker"),
        ..Default::default()
    };

    eframe::run_native(
        "Blight Tracker",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(TrackerApp::new(&cc)))
        }),
    )
}
