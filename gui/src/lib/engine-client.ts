export type ConnectionStatus = "disconnected" | "connecting" | "connected";

export interface StereoValue {
  readonly left: number;
  readonly right: number;
}

export interface StereoMeterFrame {
  /** Linear normalized peak values, one per output channel. */
  readonly peak: StereoValue;
  /** Linear normalized RMS values, one per output channel. */
  readonly rms: StereoValue;
}

export type Unsubscribe = () => void;

/**
 * The browser-facing boundary required by the current transport/gain/meter slice.
 * Host implementations own all process, network, audio-device, and filesystem work.
 */
export interface EngineClient {
  getConnectionStatus(): ConnectionStatus;

  subscribeConnectionStatus(
    listener: (status: ConnectionStatus) => void,
  ): Unsubscribe;

  play(): Promise<void>;
  stop(): Promise<void>;

  /** Writes a finite master gain in the inclusive normalized range 0...1. */
  setMasterGain(normalizedGain: number): Promise<void>;

  subscribeMeters(listener: (frame: StereoMeterFrame) => void): Unsubscribe;
}
