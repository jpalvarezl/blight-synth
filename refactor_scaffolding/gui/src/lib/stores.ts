/**
 * stores.ts
 * Svelte writable stores representing DSP parameter state.
 * Components read from these; the OscBridge writes to them on inbound echoes.
 */

import { writable, type Writable } from "svelte/store";

export interface Params {
  gain: number;
  filterCutoff: number;
  filterResonance: number;
}

export interface TransportState {
  playing: boolean;
  bpm: number;
  positionBeats: number;
}

export interface MeterState {
  levelDb: number;
  peakDb: number;
}

// TODO: initialise from a saved preset or defaults
export const params: Writable<Params> = writable({
  gain: 1.0,
  filterCutoff: 1000,
  filterResonance: 0.5,
});

export const transport: Writable<TransportState> = writable({
  playing: false,
  bpm: 120,
  positionBeats: 0,
});

export const meter: Writable<MeterState> = writable({
  levelDb: -60,
  peakDb: -60,
});

/**
 * Update a single param by key and dispatch OSC to the DSP core.
 * Import oscBridge from host context before calling.
 */
export function setParam<K extends keyof Params>(key: K, value: Params[K]): void {
  params.update((p) => ({ ...p, [key]: value }));
  // TODO: call oscBridge.sendParamSet(key, value)
}
