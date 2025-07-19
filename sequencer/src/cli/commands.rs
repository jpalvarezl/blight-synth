use crate::Result;
use crate::cli::{CliArgs, FileFormat};
use crate::models::Sequencer;

impl CliArgs {
    pub fn write_file(&self, sequencer: &Sequencer) -> Result<()> {
        let mut file = std::fs::File::create(self.output_file())?;
        match self.file_format() {
            FileFormat::Json => {
                serde_json::to_writer(file, sequencer)?;
            }
            FileFormat::Binary => {
                bincode::encode_into_std_write(sequencer, &mut file, bincode::config::standard())?;
            }
        }
        Ok(())
    }
}
