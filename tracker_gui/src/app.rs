use eframe::egui;
use sequencer::models::Song;
use sequencer::cli::FileFormat;
use sequencer::project::{write_song_to_file, open_song_from_file};

use crate::tabs::{CurrentTab, arrangement::ArrangementTab, chains::ChainsTab, phrases::PhrasesTab};

pub struct TrackerApp {
    pub song: Song,
    pub song_name: String,
    pub bpm: String,
    pub speed: String,
    pub current_tab: CurrentTab,
    
    // Tab handlers
    pub arrangement_tab: ArrangementTab,
    pub chains_tab: ChainsTab,
    pub phrases_tab: PhrasesTab,
}

impl TrackerApp {
    pub fn save_song(&mut self, format: FileFormat) {
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
    
    pub fn load_song(&mut self) {
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
                    // Reset tab states
                    self.arrangement_tab.reset();
                    self.chains_tab.reset();
                    self.phrases_tab.reset();
                    println!("Song loaded successfully from: {}", path.display());
                }
                Err(e) => {
                    eprintln!("Failed to load song: {}", e);
                }
            }
        }
    }
    
    pub fn new_song(&mut self) {
        self.song = Song::new("New Song");
        self.song_name = "New Song".to_string();
        self.bpm = "120".to_string();
        self.speed = "6".to_string();
        // Reset tab states
        self.arrangement_tab.reset();
        self.chains_tab.reset();
        self.phrases_tab.reset();
    }
    
    fn show_song_info(&mut self, ui: &mut egui::Ui) {
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
    }
    
    fn show_tab_selector(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.current_tab, CurrentTab::Arrangement, "Arrangement");
            ui.selectable_value(&mut self.current_tab, CurrentTab::Chains, "Chains");
            ui.selectable_value(&mut self.current_tab, CurrentTab::Phrases, "Phrases");
        });
    }
}

impl Default for TrackerApp {
    fn default() -> Self {
        Self {
            song: Song::new("New Song"),
            song_name: "New Song".to_string(),
            bpm: "120".to_string(),
            speed: "6".to_string(),
            current_tab: CurrentTab::Arrangement,
            arrangement_tab: ArrangementTab::default(),
            chains_tab: ChainsTab::default(),
            phrases_tab: PhrasesTab::default(),
        }
    }
}

impl eframe::App for TrackerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

            // Song info section (always visible)
            self.show_song_info(ui);
            ui.separator();

            // Tab selector
            self.show_tab_selector(ui);
            ui.separator();

            // Tab content
            match self.current_tab {
                CurrentTab::Arrangement => self.arrangement_tab.show(ui, &mut self.song),
                CurrentTab::Chains => self.chains_tab.show(ui, &mut self.song),
                CurrentTab::Phrases => self.phrases_tab.show(ui, &mut self.song),
            }
        });
    }
}
