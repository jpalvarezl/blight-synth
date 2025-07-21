use crate::Result;
use crate::cli::{CliArgs, FileFormat};
use crate::models::Song;

impl CliArgs {
    pub fn write_file(&self, song: &Song) -> Result<()> {
        let mut file = std::fs::File::create(self.output_file())?;
        match self.file_format() {
            FileFormat::Json => {
                serde_json::to_writer(file, song)?;
            }
            FileFormat::Binary => {
                bincode::encode_into_std_write(song, &mut file, bincode::config::standard())?;
            }
        }
        Ok(())
    }
}
