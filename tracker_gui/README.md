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
- **Arrangement View**: Visual grid showing the song arrangement with 8 tracks (MAX_TRACKS)
- **Hexadecimal Display**: Chain indices displayed in hexadecimal format like classic trackers
- **Interactive Editing**: 
  - Click on cells to edit chain indices
  - Use "--" or empty values to represent empty slots
  - Add and remove rows from the arrangement
- **Navigation**: Click on rows and tracks to change current position
- **Status Information**: Shows current position, arrangement length, and bank sizes

## Usage

```bash
cargo run -p tracker_gui
```

## Interface Layout

- **Menu Bar**: File menu with New, Load, Export, and Quit options
- **Song Info Section**: Editable song name, BPM, and speed fields
- **Track Headers**: Column labels for each of the 8 tracks
- **Arrangement Grid**: Scrollable grid showing song arrangement
- **Control Buttons**: Add/Remove row functionality
- **Status Bar**: Current position and bank information

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
