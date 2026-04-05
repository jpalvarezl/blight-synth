/**
 * host.ts — Bun host process
 *
 * Responsibilities:
 *   - Spawn the Rust DSP core as a child process
 *   - Own the OSC UDP sockets (send + receive)
 *   - Expose a simple internal event bus so Svelte stores can subscribe
 */

import { OscBridge } from "./src/lib/OscBridge";
import { DspProcess } from "./src/lib/DspProcess";

const DSP_BINARY = "../dsp-core/target/release/dsp-core";

async function main() {
  // TODO: parse CLI flags (port overrides, debug mode, etc.)

  const dsp = new DspProcess(DSP_BINARY);
  await dsp.start();

  const osc = new OscBridge({
    sendPort: 9000,   // DSP core listens here
    recvPort: 9001,   // we listen here for updates from DSP core
  });

  await osc.start();

  // Graceful shutdown
  process.on("SIGINT", async () => {
    await osc.stop();
    await dsp.stop();
    process.exit(0);
  });
}

main();
