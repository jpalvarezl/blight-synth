use sequencer::models::Song;
use sequencer::cli::FileFormat;
use sequencer::project::{write_song_to_file, open_song_from_file};

pub struct FileOperations;

impl FileOperations {
    pub fn save_song(song: &Song, format: FileFormat) {
        let (filter_name, extension) = match format {
            FileFormat::Json => ("JSON files", "json"),
            FileFormat::Binary => ("Binary files", "bin"),
        };
        
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(filter_name, &[extension])
            .set_file_name(&format!("{}.{}", song.name, extension))
            .save_file()
        {
            match write_song_to_file(song, &path, &format) {
                Ok(_) => {
                    println!("Song saved successfully to: {}", path.display());
                }
                Err(e) => {
                    eprintln!("Failed to save song: {}", e);
                }
            }
        }
    }
    
    pub fn load_song() -> Option<Song> {
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
                    println!("Song loaded successfully from: {}", path.display());
                    Some(song)
                }
                Err(e) => {
                    eprintln!("Failed to load song: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }
    
    pub fn new_song() -> Song {
        Song::new("New Song")
    }
}
