# DLS Loop Information Analysis

## Quick Summary

**All loop data is provided in BOTH sample frames AND seconds** for easy reference:
- **Sample frames** (loop_start, loop_length, loop_end): Exact sample indices for playback
- **Time in seconds** (loop_start_sec, loop_length_sec, loop_end_sec): Human-readable duration
- **Sample rate**: All samples use 22,050 Hz (conversion: seconds = samples ÷ 22,050)

Key findings:
- 1,895 total samples, 1,466 with loop points (77.4%)
- Median loop length: **9.1 ms** (200 samples)
- Typical loop range: 7-100 ms for sustained instruments
- Piano C2 example: Loop from 0.544s to 0.620s (76ms loop)

## File Analyzed
- **Path**: `/System/Library/Components/CoreAudio.component/Contents/Resources/gs_instruments.dls`
- **Size**: 1,996,068 bytes (approximately 1.9 MB)
- **Format**: DLS (Downloadable Sounds Level 1)
- **Sample Rate**: 22,050 Hz (all samples)

## Summary Statistics

| Metric | Count |
|--------|-------|
| Total WAV Samples | **1,895** |
| Samples WITH Loop Points | **1,466** (77.4%) |
| Samples WITHOUT Loop Points | **429** (22.6%) |

### Loop Length Statistics (for samples with loops)

| Statistic | Value |
|-----------|-------|
| Shortest loop | 0.5 ms (10 samples) - Very high note |
| Typical short loop | ~7-10 ms |
| Median loop length | **9.1 ms** (200 samples) |
| Typical range | 1 ms to 1 second |
| Longest valid loop | ~800 ms (17,648 samples) - Sound effects |

**Note**: There are a few samples with corrupted or unusual loop data (e.g., MIDI notes > 127, extremely long loops). These are edge cases and should be validated when parsing.

## Loop Information Structure

Each sample in the DLS file contains a `wsmp` (Wave Sample) chunk with the following information:

### wsmp Chunk Structure
```
Offset  Size  Field                Description
------  ----  ------------------  ----------------------------------------
+0      4     cbSize              Size of structure (usually 0x14 = 20 bytes)
+4      2     usUnityNote         MIDI note number for original pitch
+6      2     sFineTune           Fine tuning in cents (-50 to +50)
+8      4     lAttenuation        Volume attenuation in 0.1dB units
+12     4     fulOptions          Options flags
+16     4     cSampleLoops        Number of loop definitions (0 or 1)
```

### Loop Structure (if cSampleLoops > 0)
```
Offset  Size  Field                Description
------  ----  ------------------  ----------------------------------------
+0      4     cbSize              Size of loop structure (0x10 = 16 bytes)
+4      4     ulLoopType          Loop type (0 = forward loop)
+8      4     ulLoopStart         Loop start point (in samples)
+12     4     ulLoopLength        Loop length (in samples)
```

**Loop End Point** = Loop Start + Loop Length

## Key Findings

### 1. Sample Rate
All samples in the macOS DLS file use a **22,050 Hz** sample rate. This means:
- 1 sample = 0.0000454 seconds (45.4 microseconds)
- 1 second = 22,050 samples
- Time in seconds = samples ÷ 22,050

### 2. Loop Start and End Points

For samples with loops, the loop information includes:
- **Loop Start Sample**: The sample index where looping begins
- **Loop Length**: Number of samples in the loop
- **Loop End Sample**: Calculated as Start + Length

Time conversions at 22,050 Hz:
- **Loop Start Time**: loop_start ÷ 22,050 seconds
- **Loop Length Time**: loop_length ÷ 22,050 seconds
- **Loop End Time**: loop_end ÷ 22,050 seconds

### 3. Example Loop Data

Here are examples from Piano samples (all at 22,050 Hz sample rate):

| Sample | Name | MIDI Note | Loop Start | Loop End | Loop Length | Loop Start (sec) | Loop End (sec) | Loop Length (sec) |
|--------|------|-----------|------------|----------|-------------|------------------|----------------|-------------------|
| 11 | Piano 1 | 36 (C2) | 11,989 | 13,671 | 1,682 | 0.544 | 0.620 | 0.076 |
| 12 | Piano 1 | 41 (F2) | 10,830 | 13,102 | 2,272 | 0.491 | 0.594 | 0.103 |
| 21 | Piano 1 | 36 (C2) | 11,989 | 13,671 | 1,682 | 0.544 | 0.620 | 0.076 |
| 31 | Piano 1d | 36 (C2) | 11,989 | 13,671 | 1,682 | 0.544 | 0.620 | 0.076 |
| 41 | Piano 2 | 36 (C2) | 11,989 | 13,671 | 1,682 | 0.544 | 0.620 | 0.076 |

**Note**: Loop lengths are typically 50-100ms for sustained instruments, allowing smooth looping without noticeable repetition.

Here are more examples showing the range of loop lengths:

| Type | Instrument | MIDI Note | Loop Length (samples) | Loop Length (seconds) |
|------|------------|-----------|----------------------|----------------------|
| Very Short | High Piano | 103 | 155 | 0.007 sec (7ms) |
| Short | Detuned EP 1 | 49 | 160 | 0.007 sec (7ms) |
| Typical | Piano 1 | 36 | 1,682 | 0.076 sec (76ms) |
| Medium | E.Piano 2 | 66 | 3,682 | 0.167 sec (167ms) |
| Long | Contrabass | 57 | 9,710 | 0.440 sec (440ms) |

**Key Insight**: Lower notes typically have longer loop lengths (in both samples and time) because they have longer wavelengths. Higher notes can use shorter loops.

### 4. Samples Without Loops

Some percussion and effect sounds don't have loop points:
- Harp samples
- Tinkle Bell
- Steel Drums
- Woodblock
- Taiko drums
- Various percussion effects

These are typically one-shot sounds that decay naturally.

## Usage in Audio Applications

When implementing sample playback with looping:

1. **Load the sample data** from the DLS file
2. **Read the wsmp chunk** to get loop information
3. **During playback**:
   - Play normally until reaching the loop end point
   - If a note is still held, jump back to the loop start point
   - Continue looping until note release
   - Apply release envelope after note-off

### Example Pseudocode

```rust
struct LoopInfo {
    has_loop: bool,
    loop_start: u32,      // Sample index (frames)
    loop_end: u32,        // Sample index (frames)
    loop_start_sec: f32,  // Time in seconds
    loop_end_sec: f32,    // Time in seconds
}

fn play_sample(sample_data: &[f32], loop_info: &LoopInfo, position: &mut u32) {
    // Advance playback position
    *position += 1;
    
    // Check if we need to loop
    if loop_info.has_loop && *position >= loop_info.loop_end {
        *position = loop_info.loop_start;
    }
    
    // Get current sample
    sample_data[*position as usize]
}

// Calculate time-based information
fn samples_to_seconds(samples: u32, sample_rate: u32) -> f32 {
    samples as f32 / sample_rate as f32
}

fn seconds_to_samples(seconds: f32, sample_rate: u32) -> u32 {
    (seconds * sample_rate as f32) as u32
}
```

## Data Files Generated

1. **dls_loop_info.csv** - Complete CSV with all 1,895 samples and their loop information
   - Columns: 
     - `sample_num`: Sample number in file
     - `name`: Sample name (instrument)
     - `unity_note`: MIDI note number
     - `fine_tune`: Fine tuning in cents
     - `sample_rate`: Sample rate in Hz (typically 22,050 Hz)
     - `total_samples`: Total sample length in frames
     - `total_length_sec`: Total sample length in seconds
     - `has_loop`: Boolean - does this sample have loop points
     - `loop_start`: Loop start point in samples (frames)
     - `loop_start_sec`: Loop start point in seconds
     - `loop_length`: Loop length in samples (frames)
     - `loop_length_sec`: Loop length in seconds
     - `loop_end`: Loop end point in samples (frames)
     - `loop_end_sec`: Loop end point in seconds

2. **analyze_dls_loops.py** - Python script to analyze DLS files in detail

3. **extract_dls_loops.py** - Python script to extract loop data to CSV format

## Additional Metadata

Each sample also includes:
- **Unity Note**: The MIDI note number at which the sample plays at its original pitch
- **Fine Tune**: Fine tuning adjustment in cents (1/100th of a semitone)
- **Attenuation**: Volume level adjustment

## Hex Dump Example

Looking at a wsmp chunk in hex:
```
wsmp chunk header:
77 73 6d 70 24 00 00 00  14 00 00 00 24 00 fc ff
w  s  m  p  $___size___  cb_size___  note_ tune_

00 00 8f ff 00 00 00 00  01 00 00 00 10 00 00 00
attenuation_  options___  1_loop____  loop_size_

00 00 00 00 d5 2e 00 00  92 06 00 00
loop_type_  loop_start_  loop_len__
```

In this example:
- Unity Note: 0x24 = 36 (MIDI note C2)
- Fine Tune: 0xFFFC = -4 cents
- Loop Start: 0x00002ED5 = 11,989 samples
- Loop Length: 0x00000692 = 1,682 samples
- Loop End: 11,989 + 1,682 = 13,671 samples

## Conclusion

The macOS General MIDI sound bank contains **495 actual instruments** (though with multiple samples per instrument, totaling 1,895 samples). Most sustained instruments (77.4%) include loop points for continuous playback, while percussive instruments typically don't need loops.

The loop information is encoded in the `wsmp` chunk and specifies exact sample indices for the loop start and end points, allowing for seamless looping of sustained notes.
