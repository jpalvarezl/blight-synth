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
    pad.addEventListener("click", () => {
      const midiValue = midiStart + idx;
      invoke("play_midi_note", { midiValue });
      // No manual .active toggle here; let the browser handle the click animation
      pad.classList.add("active");
      setTimeout(() => pad.classList.remove("active"), 120);
    });
  });

  function triggerPad(index: number) {
    const pad = padButtons[index];
    if (!pad) return;
    pad.classList.add("active");
    pad.click();
    setTimeout(() => pad.classList.remove("active"), 120);
  }

  window.addEventListener("keydown", (e: KeyboardEvent) => {
    const active = document.activeElement;
    if (active && active.tagName === "INPUT") return;
    const key = e.key.toLowerCase();
    if (key in keyToPadIndex) {
      triggerPad(keyToPadIndex[key]);
    }
  });
}

window.addEventListener('DOMContentLoaded', setupPadKeyListeners);
