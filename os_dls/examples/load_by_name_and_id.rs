use os_dls::load_mac_os_default;

fn main() {
    println!("Loading macOS General MIDI sound bank...\n");

    let dls_file = load_mac_os_default().expect("Failed to load macOS DLS file");

    println!("✓ Loaded: {}", dls_file.path());
    println!("  Size: {} bytes\n", dls_file.size());

    // Example 1: List first 10 sample names
    println!("=== Listing first 10 samples ===");
    match dls_file.list_sample_names() {
        Ok(names) => {
            for (id, name) in names.iter().take(10) {
                println!("  [{}] {}", id, name);
            }
            println!("  ... and {} more samples\n", names.len() - 10);
        }
        Err(e) => {
            eprintln!("Failed to list samples: {}", e);
            return;
        }
    }

    // Example 2: Load a sample by name
    println!("=== Loading sample by name ===");
    let sample_name = "PIANO36";
    match dls_file.get_sample_by_name(sample_name) {
        Ok(sample) => {
            println!("✓ Found sample: {}", sample_name);
            println!("  Sample rate: {} Hz", sample.sample_rate());
            println!("  Channels: {}", sample.channels());
            println!("  Bits per sample: {}", sample.bits_per_sample());
            println!("  Audio data size: {} bytes", sample.size());
            println!(
                "  Unity note: {}",
                sample
                    .unity_note_name()
                    .unwrap_or_else(|| "N/A".to_string())
            );
            println!("  Duration: {:.2}s\n", sample.duration_seconds());
        }
        Err(e) => {
            eprintln!("Failed to load sample by name: {}", e);
        }
    }

    // Example 3: Load a sample by ID
    println!("=== Loading sample by ID ===");
    let sample_id = 289; // PIANO36 is at index 289
    match dls_file.get_sample_by_id(sample_id) {
        Ok(sample) => {
            println!(
                "✓ Found sample at ID {}: {}",
                sample_id,
                sample.name().unwrap_or("unnamed")
            );
            println!("  Sample rate: {} Hz", sample.sample_rate());
            println!("  Channels: {}", sample.channels());
            println!("  Bits per sample: {}", sample.bits_per_sample());
            println!("  Audio data size: {} bytes", sample.size());
            println!(
                "  Unity note: {}",
                sample
                    .unity_note_name()
                    .unwrap_or_else(|| "N/A".to_string())
            );
            println!("  Duration: {:.2}s\n", sample.duration_seconds());
        }
        Err(e) => {
            eprintln!("Failed to load sample by ID: {}", e);
        }
    }

    // Example 4: Try to load a non-existent sample
    println!("=== Testing error handling ===");
    match dls_file.get_sample_by_name("NONEXISTENT") {
        Ok(_) => println!("Unexpectedly found sample!"),
        Err(e) => println!("✓ Correctly handled missing sample: {}", e),
    }

    match dls_file.get_sample_by_id(9999) {
        Ok(_) => println!("Unexpectedly found sample!"),
        Err(e) => println!("✓ Correctly handled invalid ID: {}", e),
    }
}
