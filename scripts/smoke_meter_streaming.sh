#!/usr/bin/env bash
set -euo pipefail

# Smoke test for /meter/level streaming (DSP -> GUI), issue #103.
#
# Opens the system default audio output device (via dsp-core) but does NOT
# start playback, so it runs in SILENCE (no audible output). It verifies that
# the DSP core streams /meter/level to 127.0.0.1:9001 at ~30 Hz with the
# expected 4-float layout.
#
# Binds:
#   - dsp-core:      127.0.0.1:9000 (listen) / 9001 (send)
#   - meter_listen:  127.0.0.1:9001 (listen)

READY_TIMEOUT_SECONDS="${READY_TIMEOUT_SECONDS:-20}"
LISTEN_SECONDS="${LISTEN_SECONDS:-2}"
LOG_FILE="${LOG_FILE:-$(mktemp -t blight-dsp-core.XXXXXX.log)}"
export RUST_LOG="${RUST_LOG:-info}"
DSP_PID=""

cleanup() {
  if [[ -n "${DSP_PID}" ]] && kill -0 "${DSP_PID}" 2>/dev/null; then
    kill "${DSP_PID}" 2>/dev/null || true
    wait "${DSP_PID}" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

echo "Starting dsp-core (silent, no playback)..."
echo "RUST_LOG=${RUST_LOG}"
echo "Log: ${LOG_FILE}"

cargo run -p audio_backend --bin dsp-core >"${LOG_FILE}" 2>&1 &
DSP_PID="$!"

for ((i = 0; i < READY_TIMEOUT_SECONDS; i++)); do
  if grep -q '^READY$' "${LOG_FILE}"; then
    echo "dsp-core is READY"
    break
  fi

  if ! kill -0 "${DSP_PID}" 2>/dev/null; then
    echo "dsp-core exited before READY" >&2
    echo "--- dsp-core log ---" >&2
    cat "${LOG_FILE}" >&2
    exit 1
  fi

  sleep 1
done

if ! grep -q '^READY$' "${LOG_FILE}"; then
  echo "Timed out waiting for dsp-core READY after ${READY_TIMEOUT_SECONDS}s" >&2
  echo "--- dsp-core log ---" >&2
  cat "${LOG_FILE}" >&2
  exit 1
fi

echo "Listening for /meter/level for ${LISTEN_SECONDS}s..."
cargo run -p audio_backend --example meter_listen "${LISTEN_SECONDS}"

echo "Meter streaming smoke test passed."
echo "dsp-core log retained at: ${LOG_FILE}"
