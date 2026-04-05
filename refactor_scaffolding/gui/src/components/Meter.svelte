<script lang="ts">
  import { meter } from "../lib/stores";

  // TODO: subscribe to OSC meter updates and write to meter store
  // TODO: peak hold logic
</script>

<div class="meter">
  <div class="level-bar" style="height: {levelPercent($meter.levelDb)}%"></div>
  <span class="label">{$meter.levelDb.toFixed(1)} dB</span>
  <!-- TODO: peak indicator -->
</div>

<script lang="ts" context="module">
  function levelPercent(db: number): number {
    // TODO: map dB range (-60..0) to 0..100%
    return 0;
  }
</script>

<style>
  .meter {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 2rem;
    height: 8rem;
    background: #222;
    /* TODO: styling */
  }

  .level-bar {
    width: 100%;
    background: #0f0;
    /* TODO: colour gradient (green -> yellow -> red) */
  }
</style>
