#!/usr/bin/env python3

import csv
import json


# Function to read notes from a CSV file
def read_notes(file_path):
    notes = []
    with open(file_path, "r") as file:
        reader = csv.reader(file)
        for row in reader:
            # Ignore lines that start with "//"
            if not row[0].startswith("//"):
                # Split note labels that represent two notes
                note_labels = row[0].split("/")
                frequency = float(row[1])
                wavelength = float(row[2])
                for note_label in note_labels:
                    # Parse the note label into pitch, accidental, and octave
                    pitch, accidental, octave = parse_note_label(note_label)
                    # Add the note to the list
                    notes.append(
                        {
                            "note_label": note_label,
                            "pitch": pitch,
                            "accidental": accidental,
                            "octave": octave,
                            "frequency": frequency,
                            "wavelength": wavelength,
                        }
                    )
    return notes


# Function to parse a note label into pitch, accidental, and octave
def parse_note_label(note_label):
    pitch = note_label[0]
    # Determine the accidental based on the note label
    accidental = (
        "Sharp" if "#" in note_label else ("Flat" if "b" in note_label else "Natural")
    )
    octave = int(note_label[-1])
    return pitch, accidental, octave


# Function to write notes to a JSON file
def write_notes(notes, file_path):
    with open(file_path, "w") as file:
        json.dump(notes, file, indent=4)


# Read notes from the CSV file
notes = read_notes("assets/notes.csv")
# Write notes to the JSON file
write_notes(notes, "assets/notes.json")
