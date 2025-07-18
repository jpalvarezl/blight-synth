use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}


impl CliArgs {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn count(&self) -> u8 {
        self.count
    }

    pub fn parse_arguments() -> Self {
        CliArgs::parse()
    }
}