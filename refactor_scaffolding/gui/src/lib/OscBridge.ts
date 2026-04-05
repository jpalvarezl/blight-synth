/**
 * OscBridge.ts
 * Owns the UDP sockets for OSC communication with the DSP core.
 * Exposes typed send methods and an event emitter for incoming messages.
 */

import { EventEmitter } from "events";

interface OscBridgeOptions {
  sendPort: number;
  recvPort: number;
  host?: string;
}

export class OscBridge extends EventEmitter {
  private socket: Awaited<ReturnType<typeof Bun.udpSocket>> | null = null;

  constructor(private options: OscBridgeOptions) {
    super();
  }

  async start(): Promise<void> {
    // TODO: open Bun.udpSocket on options.recvPort
    // TODO: on each incoming datagram, decode OSC packet and emit typed event
  }

  async stop(): Promise<void> {
    // TODO: close the UDP socket
  }

  // --- Outbound messages to DSP core ---

  sendParamSet(paramId: string, value: number): void {
    // TODO: encode OSC message /param/set [paramId, value]
    // TODO: send datagram to 127.0.0.1:sendPort
  }

  sendTransportPlay(): void {
    // TODO: encode and send /transport/play
  }

  sendTransportStop(): void {
    // TODO: encode and send /transport/stop
  }

  sendPresetLoad(presetName: string): void {
    // TODO: encode and send /preset/load [presetName]
  }

  // --- Inbound message handlers (called from socket listener) ---

  private handleMeterUpdate(levelDb: number): void {
    // TODO: emit "meter" event with levelDb
  }

  private handleParamEcho(paramId: string, value: number): void {
    // TODO: emit "param" event so Svelte stores can sync
  }
}
