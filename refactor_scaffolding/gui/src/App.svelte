<script lang="ts">
  import Transport from "./components/Transport.svelte";
  import ParameterPanel from "./components/ParameterPanel.svelte";
  import Meter from "./components/Meter.svelte";
</script>

<main>
  <header>
    <h1>DSP Project</h1>
    <!-- TODO: preset selector -->
  </header>

  <section class="top-bar">
    <Transport />
    <Meter />
  </section>

  <section class="parameters">
    <ParameterPanel />
  </section>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
    /* TODO: design system / theme variables */
  }

  .top-bar {
    display: flex;
    align-items: center;
    gap: 1rem;
    /* TODO: styling */
  }

  .parameters {
    flex: 1;
    /* TODO: styling */
  }
</style>
