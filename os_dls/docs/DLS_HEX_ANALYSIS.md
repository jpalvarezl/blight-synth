# Detailed Hex Dump Analysis - DLS Loop Points

## Real Example from gs_instruments.dls

Here's an actual wsmp chunk from the file with detailed annotations:

```
Address   Hex Data                                          ASCII    Description
--------  ------------------------------------------------  -------  ---------------------------
00000070  77 73 6d 70 24 00 00 00  14 00 00 00 24 00 fc ff  |wsmp$.......$...|
          ^^^^^^^^^^^ ^^^^^^^^^^^  ^^^^^^^^^^^ ^^^^^ ^^^^^
          "wsmp"      Chunk Size   cbSize=20   Note  Tune
          ID (ASCII)  = 36 bytes   bytes       =36   =-4

00000080  00 00 8f ff 00 00 00 00  01 00 00 00 10 00 00 00  |................|
          ^^^^^^^^^^^ ^^^^^^^^^^^  ^^^^^^^^^^^ ^^^^^^^^^^^
          Attenuation Options=0    NumLoops=1  LoopCbSize
          = -7405568                           = 16 bytes

00000090  00 00 00 00 d5 2e 00 00  92 06 00 00 77 6c 6e 6b  |............wlnk|
          ^^^^^^^^^^^ ^^^^^^^^^^^  ^^^^^^^^^^^
          LoopType=0  LoopStart    LoopLength  Next chunk
          (forward)   = 11,989     = 1,682     starts here
```

## Detailed Field Breakdown

### Chunk Header (8 bytes)
```
Offset  Bytes        Value       Field
------  -----------  ----------  ---------------
0x70    77 73 6d 70  "wsmp"      Chunk ID (ASCII)
0x74    24 00 00 00  0x00000024  Chunk Size = 36 bytes
```

### wsmp Structure (20 bytes)
```
Offset  Bytes        Value       Field                    Interpretation
------  -----------  ----------  -----------------------  --------------------------
0x78    14 00 00 00  0x00000014  cbSize                   Structure size = 20 bytes
0x7C    24 00        0x0024      usUnityNote              MIDI Note 36 = C2
0x7E    fc ff        0xFFFC      sFineTune (signed)       -4 cents
0x80    00 00 8f ff  0xFF8F0000  lAttenuation (signed)    -7405568 (× 0.1dB)
0x84    00 00 00 00  0x00000000  fulOptions               No special options
0x88    01 00 00 00  0x00000001  cSampleLoops             1 loop defined
```

### Loop Structure (16 bytes) - Only present if cSampleLoops > 0
```
Offset  Bytes        Value       Field                    Interpretation
------  -----------  ----------  -----------------------  --------------------------
0x8C    10 00 00 00  0x00000010  cbSize                   Loop structure size = 16
0x90    00 00 00 00  0x00000000  ulLoopType               0 = Forward loop
0x94    d5 2e 00 00  0x00002ED5  ulLoopStart              11,989 samples
0x98    92 06 00 00  0x00000692  ulLoopLength             1,682 samples
```

### Calculated Loop End
```
Loop End = Loop Start + Loop Length
         = 11,989 + 1,682
         = 13,671 samples
```

## Second Example - Different Note

```
Address   Hex Data                                          ASCII    
--------  ------------------------------------------------  -------
000000d0  77 73 6d 70 24 00 00 00  14 00 00 00 29 00 ff ff  |wsmp$.......)...|
          ^^^^^^^^^^^ ^^^^^^^^^^^  ^^^^^^^^^^^ ^^^^^ ^^^^^
          "wsmp"      Size=36      cbSize=20   Note  Tune
                                              =41   =-1

000000e0  00 00 9c ff 00 00 00 00  01 00 00 00 10 00 00 00  |................|
          ^^^^^^^^^^^ ^^^^^^^^^^^  ^^^^^^^^^^^
          Attenuation Options      NumLoops=1
          
000000f0  00 00 00 00 4e 2a 00 00  e0 08 00 00 77 6c 6e 6b  |....N*......wlnk|
          ^^^^^^^^^^^ ^^^^^^^^^^^  ^^^^^^^^^^^
          LoopType=0  LoopStart    LoopLength
                      = 10,830     = 2,272
```

This sample:
- MIDI Note: 41 (F2)
- Fine Tune: -1 cent
- Loop Start: 10,830 samples
- Loop Length: 2,272 samples  
- Loop End: 13,102 samples

## Example Without Loop

Some samples have `cSampleLoops = 0`:

```
Address   Hex Data                                          ASCII    
--------  ------------------------------------------------  -------
00165490  77 73 6d 70 14 00 00 00  14 00 00 00 47 00 0c 00  |wsmp........G...|
          ^^^^^^^^^^^ ^^^^^^^^^^^  ^^^^^^^^^^^ ^^^^^ ^^^^^
          "wsmp"      Size=20      cbSize=20   Note  Tune
                      (no loop!)              =71   =12

001654a0  00 00 00 00 00 00 00 00  00 00 00 00 64 61 74 61  |............data|
          ^^^^^^^^^^^ ^^^^^^^^^^^  ^^^^^^^^^^^
          Attenuation Options      NumLoops=0  Next chunk
                                   NO LOOP!    = "data"
```

Notice:
- Chunk size is only 20 bytes (not 36)
- cSampleLoops = 0
- No loop structure follows
- Next chunk ("data") starts immediately

## Summary Table

| Component | Offset from wsmp | Size | Type | Notes |
|-----------|------------------|------|------|-------|
| Chunk ID | 0 | 4 | char[4] | "wsmp" |
| Chunk Size | 4 | 4 | uint32 | 20 (no loop) or 36 (with loop) |
| cbSize | 8 | 4 | uint32 | Always 20 |
| Unity Note | 12 | 2 | uint16 | MIDI note 0-127 |
| Fine Tune | 14 | 2 | int16 | Cents, typically -50 to +50 |
| Attenuation | 16 | 4 | int32 | Volume in 0.1 dB units |
| Options | 20 | 4 | uint32 | Usually 0 |
| Num Loops | 24 | 4 | uint32 | 0 or 1 |
| **Loop Data** (if Num Loops > 0) |
| Loop cb Size | 28 | 4 | uint32 | Always 16 |
| Loop Type | 32 | 4 | uint32 | 0 = forward |
| Loop Start | 36 | 4 | uint32 | Sample index |
| Loop Length | 40 | 4 | uint32 | Number of samples |

## Reading in Little-Endian

All multi-byte values are stored in **little-endian** format:
- `24 00` = 0x0024 = 36 (not 0x2400)
- `d5 2e 00 00` = 0x00002ED5 = 11,989 (not 0xD52E0000)

This is the standard for RIFF files and Intel/x86 architectures.

## Converting Samples to Time

All samples in the macOS DLS file use a **22,050 Hz sample rate**:

```
Time (seconds) = Samples ÷ 22,050
Samples = Time (seconds) × 22,050

Examples:
- 11,989 samples = 11,989 ÷ 22,050 = 0.544 seconds
- 1,682 samples = 1,682 ÷ 22,050 = 0.076 seconds (76 milliseconds)
- 0.1 seconds = 0.1 × 22,050 = 2,205 samples
```

### Quick Reference: Loop Lengths

| Samples | Milliseconds | Seconds | Typical Use |
|---------|--------------|---------|-------------|
| 155 | 7 ms | 0.007 | Very high notes |
| 1,000 | 45 ms | 0.045 | High notes |
| 1,682 | 76 ms | 0.076 | Piano middle C |
| 2,205 | 100 ms | 0.100 | Standard loop |
| 5,000 | 227 ms | 0.227 | Low notes |
| 9,710 | 440 ms | 0.440 | Very low notes (bass) |
| 22,050 | 1000 ms | 1.000 | 1 second |
