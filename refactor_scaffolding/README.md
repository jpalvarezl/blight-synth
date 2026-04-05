# audio-dsp-project

A modular audio DSP project with a clean separation between the DSP core,
GUI, and optional plugin wrapper.

## Architecture

```
audio-dsp-project/
├── dsp-core/       Rust — audio engine, OSC server, shared state
├── gui/            Bun + Svelte — desktop GUI, speaks OSC with dsp-core
└── plugin/         JUCE C++ — VST3/AU wrapper, embeds webview + spawns Bun
```

### Communication

```
 ┌──────────┐  OSC/UDP  ┌──────────┐  WebView  ┌──────────┐
 │ dsp-core │◄─────────►│   Bun    │◄──────────│  JUCE    │
 │  (Rust)  │           │ (host.ts)│           │ (plugin) │
 └──────────┘           └──────────┘           └──────────┘
      │                      │
  Audio I/O             Svelte UI
   (cpal)              (localhost)
```

### OSC Address Space

| Address           | Direction      | Args                  |
|-------------------|----------------|-----------------------|
| `/param/set`      | GUI → DSP      | string id, float val  |
| `/param/echo`     | DSP → GUI      | string id, float val  |
| `/transport/play` | GUI → DSP      | —                     |
| `/transport/stop` | GUI → DSP      | —                     |
| `/preset/load`    | GUI → DSP      | string name           |
| `/meter/level`    | DSP → GUI      | float db              |

## Development

### Standalone (no plugin)

```bash
# Terminal 1 — build and run DSP core
cd dsp-core
cargo run

# Terminal 2 — run GUI
cd gui
bun run dev
```

### Plugin

```bash
# Build Rust core as static library (TODO: add lib target to Cargo.toml)
cd dsp-core && cargo build --release

# Configure and build JUCE plugin
cd plugin
cmake -B build -DJUCE_PATH=/path/to/JUCE
cmake --build build
```

## Ports

| Port | Owner    | Protocol | Purpose                  |
|------|----------|----------|--------------------------|
| 9000 | dsp-core | OSC/UDP  | Receive from GUI/JUCE    |
| 9001 | Bun      | OSC/UDP  | Receive from dsp-core    |
| 5173 | Bun/Vite | HTTP     | Serve Svelte UI          |
