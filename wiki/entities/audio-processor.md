---
tags: [module, audio, rt]
sources: []
last-updated: 2025-05-11
source-file: audio_backend/src/audio_processor/mod.rs
source-sha: e6487a0519d53eab1370808414cf2a21f6eb6fd5
source-mtime: 1781291617178
last-synced: 2026-06-12
---
# audio_processor — AudioProcessor

Realtime audio processor; lives on the audio thread (driven by the cpal output callback). Holds the command-queue consumer, the `Player`, pre-allocated non-interleaved `left_buf`/`right_buf` (`MAX_BUFFER_SIZE = 4096`), and an `Arc<MeterState>`.

## process(output_buffer)

1. Drain command queue (`try_pop`, non-blocking) → `player.handle_command`.
2. `player.process(left, right, sample_rate, frame_count)` — fills post-master stereo (zero-filled when stopped).
3. **Record meter block**: `meter.record_block(left, right)` — the post-master metering point (see [[concepts/rt-nrt-metering]], [[entities/meter-state]]).
4. Re-interleave into `output_buffer`.

## Construction

- `new(command_rx, sample_rate, channels, meter)` (default Untitled song).
- `new_with_song(song, command_rx, sample_rate, channels, meter)`.

Both take an `Arc<MeterState>` (added for #103).
