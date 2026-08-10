import type {
  ConnectionStatus,
  EngineClient,
  StereoMeterFrame,
  Unsubscribe,
} from "./engine-client";

export type TransportRequest = "play" | "stop";

export interface FakeEngineClientOptions {
  connectionStatus?: ConnectionStatus;
  masterGain?: number;
  meterFrame?: StereoMeterFrame;
}

const DEFAULT_METER_FRAME: StereoMeterFrame = {
  peak: { left: 0, right: 0 },
  rms: { left: 0, right: 0 },
};

/** Deterministic in-memory client for browser development and tests. */
export class FakeEngineClient implements EngineClient {
  #connectionStatus: ConnectionStatus;
  #masterGain: number;
  #meterFrame: StereoMeterFrame;
  #connectionListeners = new Set<(status: ConnectionStatus) => void>();
  #meterListeners = new Set<(frame: StereoMeterFrame) => void>();
  #transportRequests: TransportRequest[] = [];
  #gainWrites: number[] = [];

  constructor(options: FakeEngineClientOptions = {}) {
    this.#connectionStatus = options.connectionStatus ?? "connected";
    this.#masterGain = options.masterGain ?? 0.75;
    this.#meterFrame = options.meterFrame ?? DEFAULT_METER_FRAME;
    assertNormalizedGain(this.#masterGain);
    assertMeterFrame(this.#meterFrame);
  }

  getConnectionStatus(): ConnectionStatus {
    return this.#connectionStatus;
  }

  subscribeConnectionStatus(
    listener: (status: ConnectionStatus) => void,
  ): Unsubscribe {
    this.#connectionListeners.add(listener);
    listener(this.#connectionStatus);
    return () => this.#connectionListeners.delete(listener);
  }

  async play(): Promise<void> {
    this.#transportRequests.push("play");
  }

  async stop(): Promise<void> {
    this.#transportRequests.push("stop");
  }

  async setMasterGain(normalizedGain: number): Promise<void> {
    assertNormalizedGain(normalizedGain);
    this.#masterGain = normalizedGain;
    this.#gainWrites.push(normalizedGain);
  }

  subscribeMeters(listener: (frame: StereoMeterFrame) => void): Unsubscribe {
    this.#meterListeners.add(listener);
    listener(this.#meterFrame);
    return () => this.#meterListeners.delete(listener);
  }

  /** Fake-only control used to drive deterministic connection changes. */
  setConnectionStatus(status: ConnectionStatus): void {
    this.#connectionStatus = status;
    for (const listener of this.#connectionListeners) listener(status);
  }

  /** Fake-only control used to drive deterministic meter events. */
  emitMeterFrame(frame: StereoMeterFrame): void {
    assertMeterFrame(frame);
    this.#meterFrame = frame;
    for (const listener of this.#meterListeners) listener(frame);
  }

  get masterGain(): number {
    return this.#masterGain;
  }

  get transportRequests(): readonly TransportRequest[] {
    return this.#transportRequests;
  }

  get gainWrites(): readonly number[] {
    return this.#gainWrites;
  }
}

function assertNormalizedGain(value: number): void {
  if (!Number.isFinite(value) || value < 0 || value > 1) {
    throw new RangeError(
      "master gain must be a finite normalized value from 0 to 1",
    );
  }
}

function assertMeterFrame(frame: StereoMeterFrame): void {
  for (const value of [
    frame.peak.left,
    frame.peak.right,
    frame.rms.left,
    frame.rms.right,
  ]) {
    if (!Number.isFinite(value) || value < 0 || value > 1) {
      throw new RangeError(
        "meter values must be finite normalized values from 0 to 1",
      );
    }
  }
}
