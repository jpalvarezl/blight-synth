import "./app.css";
import App from "./App.svelte";
import { FakeEngineClient } from "./lib/fake-engine-client";
import { mount } from "svelte";

const client = new FakeEngineClient({
  connectionStatus: "connected",
  masterGain: 0.75,
  meterFrame: {
    peak: { left: 0.68, right: 0.61 },
    rms: { left: 0.42, right: 0.38 },
  },
});

mount(App, {
  target: document.getElementById("app")!,
  props: { client },
});
