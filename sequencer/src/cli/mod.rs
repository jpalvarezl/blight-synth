use std::path::PathBuf;

mod commands;

use clap::{Parser, ValueEnum};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(
    version,
    about = Some("This program stores song sequences in a specified file format."))]
pub struct CliArgs {
    /// Output file to store the sequences
    #[arg(short, long)]
    output_file: PathBuf,

    /// Format of the file to store
    #[arg(short, long, value_enum, default_value_t = FileFormat::Json)]
    file_format: FileFormat,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum FileFormat {
    Json,
    Binary,
}

impl CliArgs {
    pub fn output_file(&self) -> &PathBuf {
        &self.output_file
    }

    pub fn file_format(&self) -> &FileFormat {
        &self.file_format
    }

    pub fn parse_arguments() -> Self {
        CliArgs::parse()
    }
}
