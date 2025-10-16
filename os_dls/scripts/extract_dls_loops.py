#!/usr/bin/env python3
"""
Extract loop information from DLS file and save to CSV.
"""

import struct
import csv

def find_sample_name(data, search_start, max_search=200):
    """Look backwards for INFO/INAM chunk to find sample name."""
    search_end = max(0, search_start - max_search)
    
    for i in range(search_start, search_end, -1):
        if data[i:i+4] == b'INAM':
            if i + 8 <= len(data):
                name_size = struct.unpack('<I', data[i+4:i+8])[0]
                name_data = data[i+8:i+8+name_size]
                name = name_data.rstrip(b'\x00').decode('ascii', errors='ignore')
                return name
    
    return "Unknown"

def find_sample_rate_and_length(data, wsmp_offset):
    """Find the sample rate and total sample length by looking for associated fmt and data chunks."""
    # Look backward for fmt chunk (within 1024 bytes)
    sample_rate = 22050  # Default fallback
    total_samples = 0
    
    search_start = max(0, wsmp_offset - 2048)
    
    # Search for 'fmt ' chunk
    for i in range(wsmp_offset, search_start, -1):
        if data[i:i+4] == b'fmt ':
            # fmt chunk found
            if i + 24 <= len(data):
                # Standard PCM fmt chunk layout:
                # +8: sample rate (4 bytes)
                sample_rate = struct.unpack('<I', data[i+12:i+16])[0]
                break
    
    # Search for 'data' chunk after wsmp
    for i in range(wsmp_offset, min(len(data) - 8, wsmp_offset + 200)):
        if data[i:i+4] == b'data':
            # data chunk found
            data_size = struct.unpack('<I', data[i+4:i+8])[0]
            # Assume 8-bit mono samples (1 byte per sample)
            total_samples = data_size
            break
    
    return sample_rate, total_samples

def extract_loop_info(filepath, output_csv):
    """Extract all loop information from DLS file."""
    with open(filepath, 'rb') as f:
        data = f.read()
    
    results = []
    offset = 0
    sample_num = 0
    
    while offset < len(data) - 8:
        if data[offset:offset+4] == b'wsmp':
            sample_num += 1
            chunk_size = struct.unpack('<I', data[offset+4:offset+8])[0]
            
            # Find sample name
            sample_name = find_sample_name(data, offset, 200)
            
            # Find sample rate and total length
            sample_rate, total_samples = find_sample_rate_and_length(data, offset)
            
            if chunk_size >= 20:
                # Parse wsmp header
                wsmp_offset = offset + 8
                us_unity_note = struct.unpack('<H', data[wsmp_offset+4:wsmp_offset+6])[0]
                s_fine_tune = struct.unpack('<h', data[wsmp_offset+6:wsmp_offset+8])[0]
                c_sample_loops = struct.unpack('<I', data[wsmp_offset+16:wsmp_offset+20])[0]
                
                # Parse loop data if present
                has_loop = c_sample_loops > 0
                loop_start = 0
                loop_length = 0
                loop_end = 0
                
                if has_loop and chunk_size >= 36:
                    loop_offset = wsmp_offset + 20
                    loop_start = struct.unpack('<I', data[loop_offset+8:loop_offset+12])[0]
                    loop_length = struct.unpack('<I', data[loop_offset+12:loop_offset+16])[0]
                    loop_end = loop_start + loop_length
                
                # Calculate time durations
                loop_start_sec = loop_start / sample_rate if sample_rate > 0 else 0
                loop_length_sec = loop_length / sample_rate if sample_rate > 0 else 0
                loop_end_sec = loop_end / sample_rate if sample_rate > 0 else 0
                total_length_sec = total_samples / sample_rate if sample_rate > 0 else 0
                
                results.append({
                    'sample_num': sample_num,
                    'name': sample_name,
                    'unity_note': us_unity_note,
                    'fine_tune': s_fine_tune,
                    'sample_rate': sample_rate,
                    'total_samples': total_samples,
                    'total_length_sec': round(total_length_sec, 4),
                    'has_loop': has_loop,
                    'loop_start': loop_start,
                    'loop_start_sec': round(loop_start_sec, 4),
                    'loop_length': loop_length,
                    'loop_length_sec': round(loop_length_sec, 4),
                    'loop_end': loop_end,
                    'loop_end_sec': round(loop_end_sec, 4)
                })
            
            offset += chunk_size + 8
        else:
            offset += 1
    
    # Write to CSV
    with open(output_csv, 'w', newline='') as csvfile:
        fieldnames = ['sample_num', 'name', 'unity_note', 'fine_tune', 
                     'sample_rate', 'total_samples', 'total_length_sec',
                     'has_loop', 'loop_start', 'loop_start_sec', 
                     'loop_length', 'loop_length_sec', 'loop_end', 'loop_end_sec']
        writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
        
        writer.writeheader()
        for row in results:
            writer.writerow(row)
    
    print(f"Extracted {len(results)} samples")
    print(f"Loop info saved to: {output_csv}")
    
    # Print summary
    with_loops = sum(1 for r in results if r['has_loop'])
    without_loops = len(results) - with_loops
    print(f"\nSummary:")
    print(f"  Samples with loops: {with_loops}")
    print(f"  Samples without loops: {without_loops}")
    
    # Show a few examples
    print(f"\nFirst 10 samples with loops:")
    print(f"{'#':<5} {'Name':<20} {'Note':<6} {'Loop Start':<12} {'Loop End':<12} {'Length':<10} {'Loop Sec':<10}")
    print("-" * 90)
    
    count = 0
    for r in results:
        if r['has_loop'] and count < 10:
            print(f"{r['sample_num']:<5} {r['name']:<20} {r['unity_note']:<6} "
                  f"{r['loop_start']:<12} {r['loop_end']:<12} {r['loop_length']:<10} "
                  f"{r['loop_length_sec']:<10.4f}")
            count += 1

if __name__ == "__main__":
    dls_path = "/System/Library/Components/CoreAudio.component/Contents/Resources/gs_instruments.dls"
    output_csv = "dls_loop_info.csv"
    
    extract_loop_info(dls_path, output_csv)
