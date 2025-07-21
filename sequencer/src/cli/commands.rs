use crate::Result;
use crate::cli::CliArgs;
use crate::models::Song;
use crate::project::{open_song_from_file, write_song_to_file};

impl CliArgs {
    pub fn write_file(&self, song: &Song) -> Result<()> {
        if self.output_file().is_none() {
            return Err(anyhow::anyhow!("Output file must be specified"));
        }
        let output_file = self.output_file().unwrap();
        write_song_to_file(song, output_file, self.file_format())
            .map_err(|e| anyhow::anyhow!("Failed to write song to file: {}", e))
    }

    pub fn open_file(&self) -> Result<Song> {
        if self.input_file().is_none() {
            return Err(anyhow::anyhow!("Input file must be specified"));
        }
        let input_file = self.input_file().unwrap();

        open_song_from_file(input_file, self.file_format())
            .map_err(|e| anyhow::anyhow!("Failed to open song from file: {}", e))
    }
}
