use eframe::egui;
use sequencer::models::{Song, MAX_TRACKS};
use sequencer::cli::FileFormat;
use sequencer::project::{write_song_to_file, open_song_from_file};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("Blight Tracker"),
        ..Default::default()
    };

    eframe::run_native(
        "Blight Tracker",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(TrackerApp::default()))
        }),
    )
}

struct TrackerApp {
    song: Song,
    current_row: usize,
    current_track: usize,
    song_name: String,
    bpm: String,
    speed: String,
}

impl TrackerApp {
    fn save_song(&mut self, format: FileFormat) {
        let filter_name = match format {
            FileFormat::Json => "JSON files",
            FileFormat::Binary => "Binary files",
        };
        
        let extension = match format {
            FileFormat::Json => "json",
            FileFormat::Binary => "bin",
        };
        
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(filter_name, &[extension])
            .set_file_name(&format!("{}.{}", self.song.name, extension))
            .save_file()
        {
            match write_song_to_file(&self.song, &path, &format) {
                Ok(_) => {
                    println!("Song saved successfully to: {}", path.display());
                }
                Err(e) => {
                    eprintln!("Failed to save song: {}", e);
                }
            }
        }
    }
    
    fn load_song(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON files", &["json"])
            .add_filter("Binary files", &["bin"])
            .pick_file()
        {
            // Determine format based on file extension
            let format = match path.extension().and_then(|ext| ext.to_str()) {
                Some("json") => FileFormat::Json,
                Some("bin") => FileFormat::Binary,
                _ => {
                    eprintln!("Unknown file format. Trying JSON format...");
                    FileFormat::Json
                }
            };
            
            match open_song_from_file(&path, &format) {
                Ok(song) => {
                    self.song = song;
                    self.song_name = self.song.name.clone();
                    self.bpm = self.song.initial_bpm.to_string();
                    self.speed = self.song.initial_speed.to_string();
                    self.current_row = 0;
                    self.current_track = 0;
                    println!("Song loaded successfully from: {}", path.display());
                }
                Err(e) => {
                    eprintln!("Failed to load song: {}", e);
                }
            }
        }
    }
    
    fn new_song(&mut self) {
        self.song = Song::new("New Song");
        self.song_name = "New Song".to_string();
        self.bpm = "120".to_string();
        self.speed = "6".to_string();
        self.current_row = 0;
        self.current_track = 0;
    }
}

impl Default for TrackerApp {
    fn default() -> Self {
        Self {
            song: Song::new("New Song"),
            current_row: 0,
            current_track: 0,
            song_name: "New Song".to_string(),
            bpm: "120".to_string(),
            speed: "6".to_string(),
        }
    }
}

impl eframe::App for TrackerApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Song").clicked() {
                        self.new_song();
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Load Song").clicked() {
                        self.load_song();
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Export as JSON").clicked() {
                        self.save_song(FileFormat::Json);
                        ui.close_menu();
                    }
                    
                    if ui.button("Export as Binary").clicked() {
                        self.save_song(FileFormat::Binary);
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Blight Tracker - M8 Style Interface");
            ui.separator();

            // Song info section
            ui.horizontal(|ui| {
                ui.label("Song:");
                if ui.text_edit_singleline(&mut self.song_name).changed() {
                    self.song.name = self.song_name.clone();
                }
                
                ui.separator();
                
                ui.label("BPM:");
                if ui.text_edit_singleline(&mut self.bpm).changed() {
                    if let Ok(bpm) = self.bpm.parse::<u16>() {
                        self.song.initial_bpm = bpm;
                    }
                }
                
                ui.separator();
                
                ui.label("Speed:");
                if ui.text_edit_singleline(&mut self.speed).changed() {
                    if let Ok(speed) = self.speed.parse::<u16>() {
                        self.song.initial_speed = speed;
                    }
                }
            });

            ui.separator();

            // Track headers
            ui.horizontal(|ui| {
                ui.label("Row");
                ui.separator();
                for track in 0..MAX_TRACKS {
                    ui.label(format!("Track {}", track + 1));
                    if track < MAX_TRACKS - 1 {
                        ui.separator();
                    }
                }
            });

            ui.separator();

            // Arrangement grid (song rows)
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for (row_idx, song_row) in self.song.arrangement.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            // Row number with highlighting for current row
                            let row_label = ui.selectable_label(
                                row_idx == self.current_row,
                                format!("{:02X}", row_idx)
                            );
                            if row_label.clicked() {
                                self.current_row = row_idx;
                            }
                            
                            ui.separator();
                            
                            // Chain indices for each track
                            for track in 0..MAX_TRACKS {
                                let chain_idx = &mut song_row.chain_indices[track];
                                let mut chain_text = if *chain_idx == usize::MAX {
                                    "--".to_string()
                                } else {
                                    format!("{:02X}", *chain_idx)
                                };
                                
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut chain_text)
                                        .desired_width(30.0)
                                        .font(egui::TextStyle::Monospace)
                                );
                                
                                if response.changed() {
                                    if chain_text == "--" || chain_text.is_empty() {
                                        *chain_idx = usize::MAX;
                                    } else if let Ok(parsed) = u8::from_str_radix(&chain_text, 16) {
                                        *chain_idx = parsed as usize;
                                    }
                                }
                                
                                if response.clicked() {
                                    self.current_track = track;
                                    self.current_row = row_idx;
                                }
                                
                                if track < MAX_TRACKS - 1 {
                                    ui.separator();
                                }
                            }
                        });
                    }
                });

            ui.separator();

            // Control buttons
            ui.horizontal(|ui| {
                if ui.button("Add Row").clicked() {
                    self.song.arrangement.push(Default::default());
                }
                
                if ui.button("Remove Row").clicked() && !self.song.arrangement.is_empty() {
                    if self.current_row < self.song.arrangement.len() {
                        self.song.arrangement.remove(self.current_row);
                        if self.current_row >= self.song.arrangement.len() && !self.song.arrangement.is_empty() {
                            self.current_row = self.song.arrangement.len() - 1;
                        }
                    }
                }
            });

            // Status bar
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(format!("Current: Row {:02X}, Track {}", self.current_row, self.current_track + 1));
                ui.separator();
                ui.label(format!("Arrangement Length: {}", self.song.arrangement.len()));
                ui.separator();
                ui.label(format!("Phrases: {}", self.song.phrase_bank.len()));
                ui.separator();
                ui.label(format!("Chains: {}", self.song.chain_bank.len()));
            });
        });
    }
}
