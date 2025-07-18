
// use sequencer::models::*;
use sequencer::cli::CliArgs;

fn main() {
    let args = CliArgs::parse_arguments();

    for _ in 0..args.count() {
        println!("Hello {}!", args.name());
    }
}
