use std::path::PathBuf;

mod commands;

use clap::{ArgGroup, Parser, ValueEnum};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(
    version,
    about = Some("This program loads/stores song sequences in a specified file format."),
    group(
        ArgGroup::new("file")
            .required(true)
            .args(&["output_file", "input_file"])
    ))]
pub struct CliArgs {
    /// Output file to store the sequences
    #[arg(short, long, default_value = None)]
    output_file: Option<PathBuf>,

    /// Format of the file to store
    #[arg(short, long, value_enum, default_value_t = FileFormat::Json)]
    file_format: FileFormat,

    /// Input file to read sequences from
    #[arg(short, long, default_value = None)]
    input_file: Option<PathBuf>,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum FileFormat {
    Json,
    Binary,
}

impl CliArgs {
    pub fn output_file(&self) -> Option<&PathBuf> {
        self.output_file.as_ref()
    }

    pub fn input_file(&self) -> Option<&PathBuf> {
        self.input_file.as_ref()
    }

    pub fn file_format(&self) -> &FileFormat {
        &self.file_format
    }

    pub fn parse_arguments() -> Self {
        CliArgs::parse()
    }
}
