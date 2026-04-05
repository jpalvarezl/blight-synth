<script lang="ts">
  import { transport } from "../lib/stores";

  function handlePlay() {
    // TODO: call oscBridge.sendTransportPlay()
    transport.update((t) => ({ ...t, playing: true }));
  }

  function handleStop() {
    // TODO: call oscBridge.sendTransportStop()
    transport.update((t) => ({ ...t, playing: false }));
  }
</script>

<div class="transport">
  <button on:click={handlePlay} disabled={$transport.playing}>Play</button>
  <button on:click={handleStop} disabled={!$transport.playing}>Stop</button>
  <span class="bpm">BPM: {$transport.bpm}</span>
  <!-- TODO: BPM input, position display -->
</div>

<style>
  .transport {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    /* TODO: styling */
  }
</style>
