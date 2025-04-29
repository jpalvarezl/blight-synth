import { invoke } from "@tauri-apps/api/core";

// Drum pad keyboard mapping and event handling

// Key mapping for each pad (1-16)
const padKeyMap = [
  '3', '4', '5', '6',
  'e', 'r', 't', 'y',
  'd', 'f', 'g', 'h',
  'c', 'v', 'b', 'n'
];

const keyToPadIndex: Record<string, number> = {};
padKeyMap.forEach((key, idx) => {
  keyToPadIndex[key] = idx;
});

const midiStart = 64; // Pad 1 = MIDI 64

function setupPadKeyListeners() {
  const padButtons = document.querySelectorAll<HTMLButtonElement>(".pad");

  padButtons.forEach((pad, idx) => {
    const midiValue = midiStart + idx;
    pad.addEventListener("mousedown", () => {
      invoke("play_midi_note", { midiValue });
      pad.classList.add("active");
    });
    pad.addEventListener("mouseup", () => {
      invoke("stop_midi_note");
      pad.classList.remove("active");
    });
    pad.addEventListener("mouseleave", () => {
      invoke("stop_midi_note");
      pad.classList.remove("active");
    });
    // For accessibility: touch events
    pad.addEventListener("touchstart", (e) => {
      e.preventDefault();
      invoke("play_midi_note", { midiValue });
      pad.classList.add("active");
    });
    pad.addEventListener("touchend", (e) => {
      e.preventDefault();
      invoke("stop_midi_note");
      pad.classList.remove("active");
    });
  });

  // Track which key is currently held
  let heldKey: string | null = null;

  window.addEventListener("keydown", (e: KeyboardEvent) => {
    const active = document.activeElement;
    if (active && active.tagName === "INPUT") return;
    const key = e.key.toLowerCase();
    if (key in keyToPadIndex && heldKey !== key) {
      heldKey = key;
      const idx = keyToPadIndex[key];
      const midiValue = midiStart + idx;
      invoke("play_midi_note", { midiValue });
      padButtons[idx].classList.add("active");
    }
  });

  window.addEventListener("keyup", (e: KeyboardEvent) => {
    const key = e.key.toLowerCase();
    if (key in keyToPadIndex && heldKey === key) {
      heldKey = null;
      const idx = keyToPadIndex[key];
      invoke("stop_midi_note");
      padButtons[idx].classList.remove("active");
    }
  });
}

window.addEventListener('DOMContentLoaded', setupPadKeyListeners);
