# Tracker GUI

A basic tracker-style music editor interface built with egui, inspired by the Dirtywave M8 tracker.

## Features

- **Song Management**: Edit song name, BPM, and speed (ticks per line)
- **File Operations**:
  - **New Song**: Create a fresh song with default settings
  - **Load Song**: Open existing songs from JSON or Binary format
  - **Export as JSON**: Save songs in human-readable JSON format
  - **Export as Binary**: Save songs in compact binary format using bincode
  - **Quit**: Exit the application
- **Real-time Playback**: Full audio engine integration
  - **Play/Stop**: Play songs with real-time audio output
  - **Auto-initialization**: Audio system initializes automatically when needed
  - **Multi-track**: 8 tracks with individual oscillator instruments
  - **Live Updates**: Changes to song data update playback in real-time
- **Tab-Based Interface**: Three main editing modes:
  - **Arrangement Tab**: Master song arrangement with 8 tracks (MAX_TRACKS)
  - **Chains Tab**: Edit chains (sequences of phrases)
  - **Phrases Tab**: Edit individual phrases (note sequences)
- **Hexadecimal Display**: All indices displayed in hexadecimal format like classic trackers
- **Interactive Editing**: 
  - Click on cells to edit values directly
  - Use "--" or empty values to represent empty slots
  - Add and remove items with dedicated buttons
- **Complete Tracker Workflow**: Create phrases → organize into chains → arrange in song
- **Real-time Updates**: All changes immediately update the song data structure

## Usage

```bash
cargo run -p tracker_gui
```

## Interface Layout

- **Menu Bar**: File menu with New, Load, Export, and Quit options
- **Song Info Section**: Editable song name, BPM, and speed fields (always visible)
- **Tab Selector**: Switch between Arrangement, Chains, and Phrases modes
- **Tab Content**: Context-sensitive editing interface

### Arrangement Tab
- **Track Headers**: Column labels for each of the 8 tracks
- **Arrangement Grid**: Scrollable grid showing song arrangement
- **Control Buttons**: Add/Remove row functionality
- **Status Bar**: Current position and arrangement length

### Chains Tab
- **Chain Selector**: Choose which chain to edit (displayed as hex buttons)
- **Chain Editor**: 16-step sequence of phrase indices
- **Control Buttons**: Add/Remove chain functionality
- Each chain step can reference a phrase or be empty (--)

### Phrases Tab
- **Phrase Selector**: Choose which phrase to edit (displayed as hex buttons)
- **Phrase Editor**: 16-step sequence of musical events
- **Note Column**: MIDI note values in hexadecimal
- **Volume Column**: Volume levels in hexadecimal
- **Control Buttons**: Add/Remove phrase functionality

## Controls

- **Add Row**: Adds a new empty row to the arrangement
- **Remove Row**: Removes the currently selected row
- **Text Editing**: Click on any chain index cell to edit it directly
- **Navigation**: Click on row numbers or cells to change the current position

## Data Model

The GUI creates and manipulates a `Song` struct from the sequencer crate, which includes:
- Song metadata (name, BPM, speed)
- Arrangement grid (rows × tracks matrix of chain indices)
- Banks for phrases, chains, instruments, and samples

## M8 Inspiration

The interface takes inspiration from the Dirtywave M8 tracker with:
- Clean, monospace font for data display
- Hexadecimal numbering system
- Grid-based arrangement view
- Minimal, functional design
