<script lang="ts">
  import type {
    ConnectionStatus,
    EngineClient,
    StereoMeterFrame,
  } from "./lib/engine-client";

  interface Props {
    client: EngineClient;
  }

  const EMPTY_METERS: StereoMeterFrame = {
    peak: { left: 0, right: 0 },
    rms: { left: 0, right: 0 },
  };

  let { client }: Props = $props();
  let connection = $state<ConnectionStatus>(client.getConnectionStatus());
  let meters = $state<StereoMeterFrame>(EMPTY_METERS);
  let gain = $state(0.75);
  let pending = $state(false);
  let actionMessage = $state("Ready");

  $effect(() => {
    const unsubscribeConnection = client.subscribeConnectionStatus((status) => {
      connection = status;
    });
    const unsubscribeMeters = client.subscribeMeters((frame) => {
      meters = frame;
    });

    return () => {
      unsubscribeConnection();
      unsubscribeMeters();
    };
  });

  async function requestTransport(action: "play" | "stop"): Promise<void> {
    pending = true;
    actionMessage = `${action === "play" ? "Starting" : "Stopping"}…`;
    try {
      await client[action]();
      actionMessage = action === "play" ? "Play requested" : "Stop requested";
    } catch (error) {
      actionMessage = messageFor(error);
    } finally {
      pending = false;
    }
  }

  async function updateGain(event: Event): Promise<void> {
    const input = event.currentTarget as HTMLInputElement;
    gain = Number(input.value);
    try {
      await client.setMasterGain(gain);
      actionMessage = `Master gain ${formatPercent(gain)}`;
    } catch (error) {
      actionMessage = messageFor(error);
    }
  }

  function messageFor(error: unknown): string {
    return error instanceof Error ? error.message : "Engine request failed";
  }

  function formatPercent(value: number): string {
    return `${Math.round(value * 100)}%`;
  }
</script>

<svelte:head>
  <meta
    name="description"
    content="Host-neutral Blight engine client development view"
  />
</svelte:head>

<main>
  <section class="panel" aria-labelledby="view-title">
    <header>
      <div>
        <p class="eyebrow">EngineClient boundary</p>
        <h1 id="view-title">Blight mock console</h1>
      </div>
      <output class="connection" data-status={connection} aria-live="polite">
        <span aria-hidden="true"></span>
        {connection}
      </output>
    </header>

    <div class="transport" aria-label="Transport controls">
      <button
        type="button"
        disabled={pending || connection !== "connected"}
        onclick={() => requestTransport("play")}>Play</button
      >
      <button
        class="secondary"
        type="button"
        disabled={pending || connection !== "connected"}
        onclick={() => requestTransport("stop")}>Stop</button
      >
      <output class="action-message" aria-live="polite">{actionMessage}</output>
    </div>

    <div class="control-grid">
      <section class="gain-card" aria-labelledby="gain-title">
        <div class="section-heading">
          <div>
            <p class="eyebrow">Parameter</p>
            <h2 id="gain-title">Master gain</h2>
          </div>
          <output for="master-gain">{formatPercent(gain)}</output>
        </div>
        <label for="master-gain">Normalized level</label>
        <input
          id="master-gain"
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={gain}
          oninput={updateGain}
        />
        <div class="range-labels" aria-hidden="true">
          <span>0</span><span>1</span>
        </div>
      </section>

      <section class="meter-card" aria-labelledby="meter-title">
        <div class="section-heading">
          <div>
            <p class="eyebrow">Event stream</p>
            <h2 id="meter-title">Stereo output</h2>
          </div>
        </div>

        <div class="meter-table">
          <span></span><span>Peak</span><span>RMS</span>
          <strong>Left</strong>
          <div class="meter">
            <meter
              aria-label="Left peak"
              min="0"
              max="1"
              value={meters.peak.left}
            ></meter>
            <small>{formatPercent(meters.peak.left)}</small>
          </div>
          <div class="meter">
            <meter aria-label="Left RMS" min="0" max="1" value={meters.rms.left}
            ></meter>
            <small>{formatPercent(meters.rms.left)}</small>
          </div>
          <strong>Right</strong>
          <div class="meter">
            <meter
              aria-label="Right peak"
              min="0"
              max="1"
              value={meters.peak.right}
            ></meter>
            <small>{formatPercent(meters.peak.right)}</small>
          </div>
          <div class="meter">
            <meter
              aria-label="Right RMS"
              min="0"
              max="1"
              value={meters.rms.right}
            ></meter>
            <small>{formatPercent(meters.rms.right)}</small>
          </div>
        </div>
      </section>
    </div>
  </section>
</main>
