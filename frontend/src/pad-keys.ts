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

function setupADSRControls() {
  const adsrContainer = document.createElement("div");
  adsrContainer.id = "adsr-controls";
  // Basic styling for the container - you can move this to your CSS file
  adsrContainer.style.marginTop = "20px";
  adsrContainer.style.padding = "10px";
  adsrContainer.style.border = "1px solid #ccc";
  adsrContainer.innerHTML = `
    <h4>ADSR Envelope</h4>
    <label style="display: block; margin-bottom: 5px;">
      Attack: <input type="range" id="attack" min="0" max="5" step="0.1" value="0.1"> <span id="attack-value">0.1s</span>
    </label>
    <label style="display: block; margin-bottom: 5px;">
      Decay: <input type="range" id="decay" min="0" max="5" step="0.1" value="0.1"> <span id="decay-value">0.1s</span>
    </label>
    <label style="display: block; margin-bottom: 5px;">
      Sustain: <input type="range" id="sustain" min="0" max="1" step="0.01" value="0.8"> <span id="sustain-value">0.8</span>
    </label>
    <label style="display: block; margin-bottom: 5px;">
      Release: <input type="range" id="release" min="0" max="5" step="0.1" value="0.3"> <span id="release-value">0.3s</span>
    </label>
  `;
  // Append to the placeholder div in index.html
  const placeholder = document.getElementById("adsr-controls-placeholder");
  if (placeholder) {
    placeholder.appendChild(adsrContainer);
  } else {
    console.error("ADSR controls placeholder not found! Appending to body as a fallback.");
    document.body.appendChild(adsrContainer); // Fallback
  }

  const attackInput = document.getElementById("attack") as HTMLInputElement;
  const decayInput = document.getElementById("decay") as HTMLInputElement;
  const sustainInput = document.getElementById("sustain") as HTMLInputElement;
  const releaseInput = document.getElementById("release") as HTMLInputElement;

  const attackValueDisplay = document.getElementById("attack-value") as HTMLSpanElement;
  const decayValueDisplay = document.getElementById("decay-value") as HTMLSpanElement;
  const sustainValueDisplay = document.getElementById("sustain-value") as HTMLSpanElement;
  const releaseValueDisplay = document.getElementById("release-value") as HTMLSpanElement;

  const sendADSRParamUpdate = (paramName: string, value: number) => {
    // TODO: Pass the specific parameter and its value to the audio backend
    // For example, using Tauri's invoke for a generic setter:
    // invoke("set_adsr_value", { param: paramName, value: value });
    // Or, if you have specific commands for each:
    switch (paramName) {
      case "attack": invoke("set_attack", { value }); break;
      case "decay": invoke("set_decay", { value }); break;
      case "sustain": invoke("set_sustain", { value }); break;
      case "release": invoke("set_release", { value }); break;
    }
    console.log(`ADSR Param Update - ${paramName}: ${value}`);
  };

  attackInput.addEventListener("input", () => {
    const value = parseFloat(attackInput.value);
    attackValueDisplay.textContent = `${value.toFixed(1)}s`;
    sendADSRParamUpdate("attack", value);
  });

  decayInput.addEventListener("input", () => {
    const value = parseFloat(decayInput.value);
    decayValueDisplay.textContent = `${value.toFixed(1)}s`;
    sendADSRParamUpdate("decay", value);
  });

  sustainInput.addEventListener("input", () => {
    const value = parseFloat(sustainInput.value);
    sustainValueDisplay.textContent = value.toFixed(2);
    sendADSRParamUpdate("sustain", value);
  });

  releaseInput.addEventListener("input", () => {
    const value = parseFloat(releaseInput.value);
    releaseValueDisplay.textContent = `${value.toFixed(1)}s`;
    sendADSRParamUpdate("release", value);
  });

  // Initialize display values
  const initializeADSRDisplays = () => {
    attackValueDisplay.textContent = `${parseFloat(attackInput.value).toFixed(1)}s`;
    decayValueDisplay.textContent = `${parseFloat(decayInput.value).toFixed(1)}s`;
    sustainValueDisplay.textContent = parseFloat(sustainInput.value).toFixed(2);
    releaseValueDisplay.textContent = `${parseFloat(releaseInput.value).toFixed(1)}s`;
  };
  initializeADSRDisplays();
}

window.addEventListener('DOMContentLoaded', () => {
  setupPadKeyListeners();
  setupADSRControls();
});
