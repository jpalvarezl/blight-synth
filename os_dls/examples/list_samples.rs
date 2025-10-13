use os_dls::load_mac_os_default;

fn main() {
    println!("Loading macOS General MIDI sound bank...\n");

    let dls_file = match load_mac_os_default() {
        Ok(file) => file,
        Err(e) => {
            eprintln!("✗ Failed to load DLS file: {}", e);
            std::process::exit(1);
        }
    };

    println!("✓ Loaded DLS file: {}", dls_file.path());
    println!(
        "  Size: {} bytes ({:.2} MB)\n",
        dls_file.size(),
        dls_file.size() as f64 / (1024.0 * 1024.0)
    );

    // Extract samples using the new API
    println!("Extracting samples from DLS file...\n");

    match dls_file.samples() {
        Ok(samples) => {
            println!("Found {} samples:\n", samples.len());
            println!(
                "{:<4} {:<40} {:<12} {:<8} {:<6} {:<10} {:<10}",
                "#", "Name", "Size", "Rate", "Ch", "Bits", "Unity Note"
            );
            println!("{}", "─".repeat(100));

            for (idx, sample) in samples.iter().enumerate() {
                let name = sample.name().unwrap_or("<unnamed>");
                let unity_note = sample
                    .unity_note_name()
                    .unwrap_or_else(|| "N/A".to_string());

                println!(
                    "{:<4} {:<40} {:<12} {:<8} {:<6} {:<10} {:<10}",
                    idx,
                    truncate_string(name, 40),
                    format_bytes(sample.size() as u32),
                    format!("{} Hz", sample.sample_rate()),
                    sample.channels(),
                    sample.bits_per_sample(),
                    unity_note
                );
            }

            println!("\n{}", "─".repeat(100));

            // Calculate totals
            let total_size: u64 = samples.iter().map(|s| s.size() as u64).sum();
            println!("\nTotal samples: {}", samples.len());
            println!(
                "Total sample data: {} ({:.2} MB)",
                format_bytes(total_size as u32),
                total_size as f64 / (1024.0 * 1024.0)
            );

            // Some statistics
            let named_samples = samples.iter().filter(|s| s.name().is_some()).count();
            let unnamed_samples = samples.len() - named_samples;
            println!("\nNamed samples: {}", named_samples);
            println!("Unnamed samples: {}", unnamed_samples);

            // Sample rate distribution
            let mut sample_rates: Vec<u32> = samples.iter().map(|s| s.sample_rate()).collect();
            sample_rates.sort();
            sample_rates.dedup();
            println!("\nSample rates found: {:?}", sample_rates);

            // Channel distribution
            let mono_count = samples.iter().filter(|s| s.is_mono()).count();
            let stereo_count = samples.iter().filter(|s| s.is_stereo()).count();
            println!("\nMono samples: {}", mono_count);
            println!("Stereo samples: {}", stereo_count);
        }
        Err(e) => {
            eprintln!("✗ Failed to parse DLS file: {}", e);
            std::process::exit(1);
        }
    }
}

fn format_bytes(bytes: u32) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
