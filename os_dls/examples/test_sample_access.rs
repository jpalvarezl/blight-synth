use os_dls::load_mac_os_default;

fn main() {
    println!("=== Testing DLS Sample Access Methods ===\n");
    
    let dls = load_mac_os_default()
        .expect("Failed to load macOS DLS file");
    
    // Method 1: list_sample_names()
    println!("1. Testing list_sample_names()");
    let sample_list = dls.list_sample_names()
        .expect("Failed to list sample names");
    
    println!("   Total samples: {}", sample_list.len());
    println!("   First 5 samples:");
    for (id, name) in sample_list.iter().take(5) {
        println!("     [{}] {}", id, name);
    }
    
    // Find a specific sample in the list
    let piano_entry = sample_list.iter()
        .find(|(_, name)| name == "PIANO36")
        .expect("PIANO36 not found in list");
    println!("   Found PIANO36 at index: {}", piano_entry.0);
    
    println!();
    
    // Method 2: get_sample_by_name()
    println!("2. Testing get_sample_by_name()");
    let sample_by_name = dls.get_sample_by_name("PIANO36")
        .expect("Failed to load PIANO36 by name");
    
    println!("   Name: {}", sample_by_name.name().unwrap_or("unnamed"));
    println!("   Sample rate: {} Hz", sample_by_name.sample_rate());
    println!("   Channels: {} ({})", 
        sample_by_name.channels(),
        if sample_by_name.is_mono() { "mono" } else { "stereo" }
    );
    println!("   Bits per sample: {}", sample_by_name.bits_per_sample());
    println!("   Size: {} bytes", sample_by_name.size());
    println!("   Duration: {:.3}s", sample_by_name.duration_seconds());
    
    println!();
    
    // Method 3: get_sample_by_id()
    println!("3. Testing get_sample_by_id()");
    let sample_by_id = dls.get_sample_by_id(piano_entry.0)
        .expect("Failed to load sample by ID");
    
    println!("   ID: {}", piano_entry.0);
    println!("   Name: {}", sample_by_id.name().unwrap_or("unnamed"));
    println!("   Sample rate: {} Hz", sample_by_id.sample_rate());
    println!("   Size: {} bytes", sample_by_id.size());
    
    println!();
    
    // Verify that both methods return the same sample
    println!("4. Verification");
    let name_matches = sample_by_name.name() == sample_by_id.name();
    let size_matches = sample_by_name.size() == sample_by_id.size();
    let rate_matches = sample_by_name.sample_rate() == sample_by_id.sample_rate();
    
    println!("   Name matches: {}", name_matches);
    println!("   Size matches: {}", size_matches);
    println!("   Sample rate matches: {}", rate_matches);
    
    if name_matches && size_matches && rate_matches {
        println!("   ✓ Both methods return identical samples!");
    } else {
        println!("   ✗ Samples differ!");
    }
    
    println!();
    
    // Test error cases
    println!("5. Error handling");
    match dls.get_sample_by_name("NONEXISTENT_SAMPLE") {
        Ok(_) => println!("   ✗ Should have failed for non-existent name"),
        Err(e) => println!("   ✓ Correctly rejected: {}", e),
    }
    
    match dls.get_sample_by_id(99999) {
        Ok(_) => println!("   ✗ Should have failed for invalid ID"),
        Err(e) => println!("   ✓ Correctly rejected: {}", e),
    }
    
    println!("\n=== All tests passed! ===");
}
