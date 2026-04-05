<script lang="ts">
  import { params, setParam } from "../lib/stores";

  function onGainChange(e: Event) {
    const value = parseFloat((e.target as HTMLInputElement).value);
    setParam("gain", value);
  }

  function onCutoffChange(e: Event) {
    const value = parseFloat((e.target as HTMLInputElement).value);
    setParam("filterCutoff", value);
  }

  function onResonanceChange(e: Event) {
    const value = parseFloat((e.target as HTMLInputElement).value);
    setParam("filterResonance", value);
  }
</script>

<div class="parameter-panel">
  <label>
    Gain
    <input
      type="range" min="0" max="2" step="0.01"
      value={$params.gain}
      on:input={onGainChange}
    />
    {$params.gain.toFixed(2)}
  </label>

  <label>
    Filter Cutoff
    <input
      type="range" min="20" max="20000" step="1"
      value={$params.filterCutoff}
      on:input={onCutoffChange}
    />
    {$params.filterCutoff} Hz
  </label>

  <label>
    Resonance
    <input
      type="range" min="0" max="1" step="0.01"
      value={$params.filterResonance}
      on:input={onResonanceChange}
    />
    {$params.filterResonance.toFixed(2)}
  </label>

  <!-- TODO: replace range inputs with proper knob components -->
</div>

<style>
  .parameter-panel {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem;
    /* TODO: styling */
  }

  label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
</style>
