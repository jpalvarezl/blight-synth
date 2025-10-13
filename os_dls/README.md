# os_dls

Extract audio samples from macOS DLS (Downloadable Sounds) files.

## Usage

```rust
use os_dls::load_mac_os_default;

let dls = load_mac_os_default()?;

// List all samples
let samples = dls.list_sample_names()?;
println!("Found {} samples", samples.len());

// Load by name
let piano = dls.get_sample_by_name("PIANO36")?;
println!("{} Hz, {} bits", piano.sample_rate(), piano.bits_per_sample());

// Load by ID
let first = dls.get_sample_by_id(0)?;

// Get all samples
let all = dls.samples()?;
```

## What it does

Parses `/System/Library/Components/CoreAudio.component/Contents/Resources/gs_instruments.dls` on macOS and extracts the 495 individual audio samples for playback.
