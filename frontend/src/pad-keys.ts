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

function setupPadKeyListeners() {
  const padButtons = document.querySelectorAll<HTMLButtonElement>('.pad');

  // Add click event listeners to each pad
  padButtons.forEach((pad, idx) => {
    pad.addEventListener('click', () => {
      // TODO: Replace this with actual sound or backend call
      console.log(`Pad ${idx + 1} pressed (keyboard: ${padKeyMap[idx]})`);
      // No manual .active toggle here; let the browser handle the click animation
    });
  });

  // Visual feedback for pad press from keyboard
  function triggerPad(index: number) {
    const pad = padButtons[index];
    if (!pad) return;
    pad.classList.add('active');
    pad.click();
    setTimeout(() => pad.classList.remove('active'), 120);
  }

  // Listen for keydown events
  window.addEventListener('keydown', (e: KeyboardEvent) => {
    const active = document.activeElement;
    if (active && active.tagName === 'INPUT') return;
    const key = e.key.toLowerCase();
    if (key in keyToPadIndex) {
      triggerPad(keyToPadIndex[key]);
    }
  });
}

window.addEventListener('DOMContentLoaded', setupPadKeyListeners);
