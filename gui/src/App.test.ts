import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import App from "./App.svelte";
import { FakeEngineClient } from "./lib/fake-engine-client";

const INITIAL_METERS = {
  peak: { left: 0.68, right: 0.61 },
  rms: { left: 0.42, right: 0.38 },
} as const;

describe("mock EngineClient view", () => {
  it("renders injected connection and stereo meter events", async () => {
    const client = new FakeEngineClient({
      connectionStatus: "connected",
      meterFrame: INITIAL_METERS,
    });
    render(App, { props: { client } });

    expect(screen.getByText("connected")).toBeInTheDocument();
    expect(meter("Left peak").value).toBe(0.68);
    expect(meter("Right peak").value).toBe(0.61);
    expect(meter("Left RMS").value).toBe(0.42);
    expect(meter("Right RMS").value).toBe(0.38);

    client.emitMeterFrame({
      peak: { left: 0.9, right: 0.82 },
      rms: { left: 0.5, right: 0.47 },
    });

    await waitFor(() => expect(meter("Left peak").value).toBe(0.9));
    expect(screen.getByText("82%")).toBeInTheDocument();
  });

  it("sends play, stop, and normalized gain requests to the injected client", async () => {
    const client = new FakeEngineClient();
    render(App, { props: { client } });

    await fireEvent.click(screen.getByRole("button", { name: "Play" }));
    await fireEvent.click(screen.getByRole("button", { name: "Stop" }));
    await fireEvent.input(screen.getByLabelText("Normalized level"), {
      target: { value: "0.34" },
    });

    await waitFor(() => {
      expect(client.transportRequests).toEqual(["play", "stop"]);
      expect(client.gainWrites).toEqual([0.34]);
    });
    expect(screen.getByText("Master gain 34%")).toBeInTheDocument();
  });

  it("disables transport while disconnected and reacts to connection events", async () => {
    const client = new FakeEngineClient({ connectionStatus: "disconnected" });
    render(App, { props: { client } });
    const play = screen.getByRole("button", { name: "Play" });
    const stop = screen.getByRole("button", { name: "Stop" });

    expect(play).toBeDisabled();
    expect(stop).toBeDisabled();

    client.setConnectionStatus("connected");

    await waitFor(() => expect(play).toBeEnabled());
    expect(stop).toBeEnabled();
  });
});

function meter(name: string): HTMLMeterElement {
  return screen.getByLabelText(name) as HTMLMeterElement;
}
