#!/usr/bin/env python3
"""
Parse DLS file and extract loop information for each sample.

DLS (Downloadable Sounds) Format Structure:
- RIFF container
- Contains wave pools (wvpl) with samples
- Each sample has a wsmp (wave sample) chunk containing loop data

wsmp chunk structure (after chunk ID and size):
- cbSize (4 bytes): Size of structure (usually 0x14 = 20 bytes)
- usUnityNote (2 bytes): MIDI note for playback at original sample rate
- sFineTune (2 bytes): Fine tuning in cents
- lAttenuation (4 bytes): Attenuation in 0.1dB units
- fulOptions (4 bytes): Options flags
- cSampleLoops (4 bytes): Number of sample loops (usually 0 or 1)
  
If cSampleLoops > 0, followed by loop structures (16 bytes each):
- cbSize (4 bytes): Size of loop structure (usually 0x10 = 16 bytes)
- ulLoopType (4 bytes): Loop type (0 = forward)
- ulLoopStart (4 bytes): Loop start point in samples
- ulLoopLength (4 bytes): Loop length in samples
"""

import struct
import sys

def read_chunk(data, offset):
    """Read a RIFF chunk header."""
    if offset + 8 > len(data):
        return None, None, offset
    
    chunk_id = data[offset:offset+4].decode('ascii', errors='ignore')
    chunk_size = struct.unpack('<I', data[offset+4:offset+8])[0]
    return chunk_id, chunk_size, offset + 8

def parse_wsmp_chunk(data, offset, size):
    """Parse a wsmp (wave sample) chunk to extract loop information."""
    if size < 20:
        return None
    
    # Parse wsmp header
    cb_size = struct.unpack('<I', data[offset:offset+4])[0]
    us_unity_note = struct.unpack('<H', data[offset+4:offset+6])[0]
    s_fine_tune = struct.unpack('<h', data[offset+6:offset+8])[0]
    l_attenuation = struct.unpack('<i', data[offset+8:offset+12])[0]
    ful_options = struct.unpack('<I', data[offset+12:offset+16])[0]
    c_sample_loops = struct.unpack('<I', data[offset+16:offset+20])[0]
    
    loop_info = {
        'unity_note': us_unity_note,
        'fine_tune': s_fine_tune,
        'attenuation': l_attenuation,
        'options': ful_options,
        'num_loops': c_sample_loops,
        'loops': []
    }
    
    # Parse loop structures if present
    if c_sample_loops > 0 and size >= 20 + 16:
        loop_offset = offset + 20
        for i in range(c_sample_loops):
            if loop_offset + 16 <= offset + size:
                loop_cb_size = struct.unpack('<I', data[loop_offset:loop_offset+4])[0]
                loop_type = struct.unpack('<I', data[loop_offset+4:loop_offset+8])[0]
                loop_start = struct.unpack('<I', data[loop_offset+8:loop_offset+12])[0]
                loop_length = struct.unpack('<I', data[loop_offset+12:loop_offset+16])[0]
                
                loop_info['loops'].append({
                    'type': loop_type,
                    'start': loop_start,
                    'length': loop_length,
                    'end': loop_start + loop_length
                })
                
                loop_offset += 16
    
    return loop_info

def find_sample_name(data, search_start, max_search=1024):
    """Look backwards for INFO/INAM chunk to find sample name."""
    # Search backwards for "INAM" chunk
    search_end = max(0, search_start - max_search)
    
    for i in range(search_start, search_end, -1):
        if data[i:i+4] == b'INAM':
            # Found INAM chunk, read its size
            if i + 8 <= len(data):
                name_size = struct.unpack('<I', data[i+4:i+8])[0]
                name_data = data[i+8:i+8+name_size]
                # Remove null terminator and decode
                name = name_data.rstrip(b'\x00').decode('ascii', errors='ignore')
                return name
    
    return "Unknown"

def parse_dls_file(filepath):
    """Parse DLS file and extract all sample loop information."""
    with open(filepath, 'rb') as f:
        data = f.read()
    
    # Verify RIFF header
    if data[0:4] != b'RIFF':
        print("Not a RIFF file!")
        return
    
    if data[8:12] != b'DLS ':
        print("Not a DLS file!")
        return
    
    print(f"DLS File Size: {len(data)} bytes")
    print(f"=" * 80)
    
    # Find all wsmp chunks
    offset = 0
    sample_num = 0
    samples_with_loops = 0
    samples_without_loops = 0
    
    while offset < len(data) - 8:
        # Look for wsmp chunks
        if data[offset:offset+4] == b'wsmp':
            sample_num += 1
            chunk_size = struct.unpack('<I', data[offset+4:offset+8])[0]
            
            # Try to find sample name
            sample_name = find_sample_name(data, offset, 200)
            
            # Parse wsmp chunk
            wsmp_info = parse_wsmp_chunk(data, offset + 8, chunk_size)
            
            if wsmp_info:
                has_loop = wsmp_info['num_loops'] > 0
                
                if has_loop:
                    samples_with_loops += 1
                else:
                    samples_without_loops += 1
                
                print(f"\nSample #{sample_num}: {sample_name}")
                print(f"  Unity Note: {wsmp_info['unity_note']} (MIDI note)")
                print(f"  Fine Tune: {wsmp_info['fine_tune']} cents")
                print(f"  Attenuation: {wsmp_info['attenuation']} (0.1 dB units)")
                print(f"  Number of Loops: {wsmp_info['num_loops']}")
                
                if wsmp_info['loops']:
                    for i, loop in enumerate(wsmp_info['loops']):
                        print(f"  Loop {i+1}:")
                        print(f"    Type: {loop['type']} (0=forward)")
                        print(f"    Start Sample: {loop['start']}")
                        print(f"    Length: {loop['length']} samples")
                        print(f"    End Sample: {loop['end']}")
            
            offset += chunk_size + 8
        else:
            offset += 1
    
    print(f"\n{'=' * 80}")
    print(f"Total samples found: {sample_num}")
    print(f"Samples with loops: {samples_with_loops}")
    print(f"Samples without loops: {samples_without_loops}")

if __name__ == "__main__":
    dls_path = "/System/Library/Components/CoreAudio.component/Contents/Resources/gs_instruments.dls"
    
    if len(sys.argv) > 1:
        dls_path = sys.argv[1]
    
    print(f"Analyzing DLS file: {dls_path}\n")
    parse_dls_file(dls_path)
