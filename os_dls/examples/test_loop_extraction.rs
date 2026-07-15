use os_dls::DlsFile;

fn main() -> Result<(), String> {
    println!("Testing DLS loop information extraction\n");

    // Load the DLS file
    let dls = DlsFile::open_macos_default()?;

    // Extract samples
    let samples = dls.samples()?;

    println!("Total samples: {}", samples.len());

    // Count samples with loop information
    let looped_samples: Vec<_> = samples.iter().filter(|s| s.has_loop()).collect();

    println!("Samples with loop info: {}\n", looped_samples.len());

    // Show first 20 samples with loop information
    println!("First 20 samples with loop information:");
    println!("{:-<120}", "");
    println!(
        "{:<5} {:<30} {:<15} {:<15} {:<15} {:<15} {:<10}",
        "#", "Name", "Loop Start", "Loop End", "Loop Length", "Loop Type", "Total Samples"
    );
    println!("{:-<120}", "");

    for (i, sample) in looped_samples.iter().take(20).enumerate() {
        if let Some(loop_info) = sample.loop_info() {
            let total_samples = sample.audio_data().len() / (sample.bits_per_sample() as usize / 8);
            println!(
                "{:<5} {:<30} {:<15} {:<15} {:<15} {:<15} {:<10}",
                i,
                sample.name().unwrap_or("<unnamed>"),
                loop_info.start,
                loop_info.end,
                loop_info.length(),
                loop_info.loop_type,
                total_samples
            );
        }
    }

    println!("{:-<120}", "");

    // Show some statistics
    if let Some(first_looped) = looped_samples.first()
        && let Some(loop_info) = first_looped.loop_info()
    {
        println!(
            "\nExample loop information (sample: {}):",
            first_looped.name().unwrap_or("<unnamed>")
        );
        println!(
            "  Loop start: {} frames ({:.3} seconds)",
            loop_info.start,
            loop_info.start_seconds(first_looped.sample_rate())
        );
        println!(
            "  Loop end: {} frames ({:.3} seconds)",
            loop_info.end,
            loop_info.end_seconds(first_looped.sample_rate())
        );
        println!(
            "  Loop length: {} frames ({:.3} seconds)",
            loop_info.length(),
            loop_info.length_seconds(first_looped.sample_rate())
        );
        println!("  Loop type: {}", loop_info.loop_type);
    }

    Ok(())
}
