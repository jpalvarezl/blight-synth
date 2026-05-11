#!/usr/bin/env bash
set -euo pipefail

# Manual smoke test for the standalone OSC loop.
#
# This one DOES open the system default audio output device and binds:
#   - dsp-core:     127.0.0.1:9000
#   - osc_control:  127.0.0.1:9001
#
# It starts dsp-core, waits for READY, sends OSC messages via the example,
# then shuts dsp-core down.

READY_TIMEOUT_SECONDS="${READY_TIMEOUT_SECONDS:-20}"
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

echo "Starting dsp-core..."
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

echo "Running OSC control example..."
cargo run -p audio_backend --example osc_control

echo "OSC smoke test completed."
echo "dsp-core log retained at: ${LOG_FILE}"
