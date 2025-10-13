use std::io::Cursor;
use riff::{Chunk, ChunkId};

use crate::sample::Sample;

const WAVE_ID: ChunkId = ChunkId { value: *b"wave" };
const FMT_ID: ChunkId = ChunkId { value: *b"fmt " };
const DATA_ID: ChunkId = ChunkId { value: *b"data" };
const WSMP_ID: ChunkId = ChunkId { value: *b"wsmp" };
const INFO_ID: ChunkId = ChunkId { value: *b"INFO" };
const INAM_ID: ChunkId = ChunkId { value: *b"INAM" };

/// Parses a DLS file and extracts sample information
pub struct DlsParser {
    data: Vec<u8>,
}

impl DlsParser {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Extract all samples from the DLS file with their audio data
    pub fn extract_samples(&self) -> Result<Vec<Sample>, String> {
        let mut cursor = Cursor::new(&self.data);
        
        // Read the root RIFF chunk
        let root_chunk = Chunk::read(&mut cursor, 0)
            .map_err(|e| format!("Failed to read root chunk: {}", e))?;
        
        // Verify it's a DLS file
        let dls_type = root_chunk.read_type(&mut cursor)
            .map_err(|e| format!("Failed to read DLS type: {}", e))?;
        
        if dls_type.value != *b"DLS " {
            return Err(format!("Not a DLS file, found type: {:?}", String::from_utf8_lossy(&dls_type.value)));
        }
        
        // Collect all chunks first to avoid borrow checker issues
        let root_children: Vec<Chunk> = root_chunk.iter(&mut cursor)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to iterate root chunks: {}", e))?;
        
        // Find the wave pool LIST chunk
        let mut samples = Vec::new();
        
        for chunk in &root_children {
            // Check if this is a LIST chunk with type 'wvpl'
            if chunk.id() == riff::LIST_ID {
                let list_type = chunk.read_type(&mut cursor)
                    .map_err(|e| format!("Failed to read LIST type: {}", e))?;
                
                if list_type.value == *b"wvpl" {
                    // Found the wave pool, collect wave chunks
                    let wave_chunks: Vec<Chunk> = chunk.iter(&mut cursor)
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| format!("Failed to iterate wave chunks: {}", e))?;
                    
                    for wave_chunk in &wave_chunks {
                        if wave_chunk.id() == riff::LIST_ID {
                            let wave_type = wave_chunk.read_type(&mut cursor)
                                .map_err(|e| format!("Failed to read wave type: {}", e))?;
                            
                            if wave_type == WAVE_ID {
                                if let Ok(sample) = self.parse_wave_chunk(wave_chunk, &mut cursor) {
                                    samples.push(sample);
                                }
                            }
                        }
                    }
                    break;
                }
            }
        }
        
        Ok(samples)
    }
    
    /// Parse a single wave chunk and extract the sample
    fn parse_wave_chunk(&self, wave_chunk: &Chunk, cursor: &mut Cursor<&Vec<u8>>) -> Result<Sample, String> {
        let mut name: Option<String> = None;
        let mut sample_rate: u32 = 0;
        let mut channels: u16 = 0;
        let mut bits_per_sample: u16 = 0;
        let mut unity_note: Option<u8> = None;
        let mut fine_tune: Option<i16> = None;
        let mut audio_data: Vec<u8> = Vec::new();
        
        // Collect subchunks first
        let subchunks: Vec<Chunk> = wave_chunk.iter(cursor)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to iterate wave subchunks: {}", e))?;
        
        // Process each subchunk
        for subchunk in &subchunks {
            match subchunk.id() {
                FMT_ID => {
                    // Parse format chunk
                    let fmt_data = subchunk.read_contents(cursor)
                        .map_err(|e| format!("Failed to read fmt chunk: {}", e))?;
                    
                    if fmt_data.len() >= 16 {
                        channels = u16::from_le_bytes([fmt_data[2], fmt_data[3]]);
                        sample_rate = u32::from_le_bytes([fmt_data[4], fmt_data[5], fmt_data[6], fmt_data[7]]);
                        bits_per_sample = u16::from_le_bytes([fmt_data[14], fmt_data[15]]);
                    }
                }
                DATA_ID => {
                    // Read audio data
                    audio_data = subchunk.read_contents(cursor)
                        .map_err(|e| format!("Failed to read data chunk: {}", e))?;
                }
                WSMP_ID => {
                    // Wave sample chunk - contains unity note and fine tune
                    let wsmp_data = subchunk.read_contents(cursor)
                        .map_err(|e| format!("Failed to read wsmp chunk: {}", e))?;
                    
                    if wsmp_data.len() >= 20 {
                        unity_note = Some(wsmp_data[16]);
                        fine_tune = Some(i16::from_le_bytes([wsmp_data[18], wsmp_data[19]]));
                    }
                }
                id if id == riff::LIST_ID => {
                    // Check if it's an INFO list
                    let list_type = subchunk.read_type(cursor)
                        .map_err(|e| format!("Failed to read LIST type: {}", e))?;
                    
                    if list_type == INFO_ID {
                        // Collect INFO chunks
                        let info_chunks: Vec<Chunk> = subchunk.iter(cursor)
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|e| format!("Failed to iterate INFO subchunks: {}", e))?;
                        
                        for info_chunk in &info_chunks {
                            if info_chunk.id() == INAM_ID {
                                let name_data = info_chunk.read_contents(cursor)
                                    .map_err(|e| format!("Failed to read name: {}", e))?;
                                
                                // Remove null terminator
                                let name_str = if let Some(null_pos) = name_data.iter().position(|&b| b == 0) {
                                    &name_data[..null_pos]
                                } else {
                                    &name_data
                                };
                                
                                name = Some(String::from_utf8_lossy(name_str).to_string());
                            }
                        }
                    }
                }
                _ => {
                    // Skip unknown chunks
                }
            }
        }
        
        Ok(Sample::new(
            name,
            audio_data,
            sample_rate,
            channels,
            bits_per_sample,
            unity_note,
            fine_tune,
        ))
    }
}
