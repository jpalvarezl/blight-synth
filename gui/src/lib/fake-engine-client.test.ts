import { describe, expect, it, vi } from "vitest";
import { FakeEngineClient } from "./fake-engine-client";

const FRAME = {
  peak: { left: 0.8, right: 0.7 },
  rms: { left: 0.45, right: 0.4 },
} as const;

describe("FakeEngineClient", () => {
  it("records current-slice transport and normalized gain requests", async () => {
    const client = new FakeEngineClient({ masterGain: 0.5 });

    await client.play();
    await client.stop();
    await client.setMasterGain(0.32);

    expect(client.transportRequests).toEqual(["play", "stop"]);
    expect(client.gainWrites).toEqual([0.32]);
    expect(client.masterGain).toBe(0.32);
  });

  it.each([-0.01, 1.01, Number.NaN, Number.POSITIVE_INFINITY])(
    "rejects an invalid normalized gain of %s",
    async (gain) => {
      const client = new FakeEngineClient();

      await expect(client.setMasterGain(gain)).rejects.toThrow(RangeError);
      expect(client.gainWrites).toEqual([]);
    },
  );

  it("publishes deterministic connection and stereo meter events", () => {
    const client = new FakeEngineClient({ connectionStatus: "disconnected" });
    const connectionListener = vi.fn();
    const meterListener = vi.fn();

    const unsubscribeConnection =
      client.subscribeConnectionStatus(connectionListener);
    const unsubscribeMeter = client.subscribeMeters(meterListener);

    expect(connectionListener).toHaveBeenLastCalledWith("disconnected");
    client.setConnectionStatus("connected");
    client.emitMeterFrame(FRAME);
    expect(connectionListener).toHaveBeenLastCalledWith("connected");
    expect(meterListener).toHaveBeenLastCalledWith(FRAME);

    unsubscribeConnection();
    unsubscribeMeter();
    client.setConnectionStatus("connecting");
    client.emitMeterFrame({
      peak: { left: 0, right: 0 },
      rms: { left: 0, right: 0 },
    });

    expect(connectionListener).toHaveBeenCalledTimes(2);
    expect(meterListener).toHaveBeenCalledTimes(2);
  });

  it("rejects invalid fake meter events without notifying listeners", () => {
    const client = new FakeEngineClient();
    const listener = vi.fn();
    client.subscribeMeters(listener);

    expect(() =>
      client.emitMeterFrame({
        peak: { left: 1.1, right: 0 },
        rms: { left: 0, right: 0 },
      }),
    ).toThrow(RangeError);
    expect(listener).toHaveBeenCalledTimes(1);
  });
});
