// Oscillator waveform dropdown control for frontend
import { invoke } from "@tauri-apps/api/core";

export const waveforms = [
  { value: "Sine", label: "Sine" },
  { value: "Square", label: "Square" },
  { value: "Sawtooth", label: "Sawtooth" },
  { value: "Triangle", label: "Triangle" }
];

export function setupOscillatorDropdown() {
  const container = document.createElement("div");
  container.style.margin = "1rem auto";
  container.style.display = "flex";
  container.style.justifyContent = "center";
  container.style.alignItems = "center";
  container.style.gap = "0.5rem";

  const label = document.createElement("label");
  label.textContent = "Oscillator Waveform:";
  label.setAttribute("for", "oscillator-waveform");
  label.style.fontWeight = "bold";

  const select = document.createElement("select");
  select.id = "oscillator-waveform";
  select.style.fontSize = "1rem";
  select.style.padding = "0.3em 0.8em";
  select.style.borderRadius = "8px";
  select.style.marginLeft = "0.5em";

  waveforms.forEach(({ value, label: lbl }) => {
    const option = document.createElement("option");
    option.value = value;
    option.textContent = lbl;
    select.appendChild(option);
  });

  select.addEventListener("change", async (e) => {
    const waveform = (e.target as HTMLSelectElement).value;
    await invoke("set_waveform", { waveform });
  });

  container.appendChild(label);
  container.appendChild(select);

  // Insert into static placeholder for robustness
  const placeholder = document.getElementById("oscillator-dropdown-placeholder");
  if (placeholder) {
    // Remove any previous dropdowns to avoid duplicates
    placeholder.innerHTML = "";
    placeholder.appendChild(container);
  }
}
