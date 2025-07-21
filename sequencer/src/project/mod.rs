use std::path::PathBuf;

use crate::Result;
use crate::cli::FileFormat;
use crate::models::Song;

pub fn new_project() -> Result<Song> {
    // Function to create a new project
    Ok(Song::new("New song"))
}

/// Writes a song to a file in the specified format
pub fn write_song_to_file(
    song: &Song,
    output_file: &PathBuf,
    file_format: &FileFormat,
) -> Result<()> {
    let mut file = std::fs::File::create(output_file)?;
    match file_format {
        FileFormat::Json => {
            serde_json::to_writer(file, song)?;
        }
        FileFormat::Binary => {
            bincode::encode_into_std_write(song, &mut file, bincode::config::standard())?;
        }
    }
    Ok(())
}

pub fn open_song_from_file(input_file: &PathBuf, file_format: &FileFormat) -> Result<Song> {
    let mut file = std::fs::File::open(input_file)?;
    match file_format {
        FileFormat::Json => {
            let song: Song = serde_json::from_reader(&mut file)?;
            Ok(song)
        }
        FileFormat::Binary => {
            let song: Song = bincode::decode_from_std_read(&mut file, bincode::config::standard())?;
            Ok(song)
        }
    }
}
