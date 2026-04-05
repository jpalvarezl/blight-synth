/**
 * DspProcess.ts
 * Spawns and supervises the Rust DSP core binary.
 */

export class DspProcess {
  private proc: ReturnType<typeof Bun.spawn> | null = null;

  constructor(private binaryPath: string) {}

  async start(): Promise<void> {
    // TODO: spawn binaryPath with Bun.spawn
    // TODO: pipe stdout/stderr to console with a [dsp] prefix
    // TODO: wait for readiness signal (e.g. "READY" line on stdout)
    // TODO: set up auto-restart on unexpected exit
  }

  async stop(): Promise<void> {
    // TODO: send SIGTERM to this.proc
    // TODO: wait for exit with a timeout, then SIGKILL
  }

  isRunning(): boolean {
    // TODO: return true if proc is alive
    return false;
  }
}
