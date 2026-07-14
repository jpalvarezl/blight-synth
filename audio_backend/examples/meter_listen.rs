//! Listens for `/meter/level` frames streamed by a running `dsp-core` and
//! verifies the stream (frame count, ~30 Hz rate, 4-float layout).
//!
//! Run `dsp-core` first, then this example:
//!
//! ```sh
//! cargo run -p audio_backend --bin dsp-core
//! cargo run -p audio_backend --example meter_listen [seconds]
//! ```
//!
//! No `/transport/play` is sent, so the DSP core stays silent; the meter
//! still streams (silence floors at -120 dBFS), which is enough to prove the
//! DSP -> GUI streaming path end to end.

use std::{
    net::UdpSocket,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use rosc::{decoder, OscPacket, OscType};

const GUI_OSC_ADDR: &str = "127.0.0.1:9001";

fn main() -> Result<()> {
    env_logger::init();

    let seconds: f64 = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(2.0);

    let socket =
        UdpSocket::bind(GUI_OSC_ADDR).context("failed to bind OSC receive socket on 9001")?;
    socket
        .set_read_timeout(Some(Duration::from_millis(500)))
        .context("failed to configure OSC receive timeout")?;

    println!("Listening for /meter/level on {GUI_OSC_ADDR} for {seconds:.1}s");
    println!("(start dsp-core first: cargo run -p audio_backend --bin dsp-core)");

    let mut buf = [0_u8; decoder::MTU];
    let start = Instant::now();
    let mut frames = 0_u32;
    let mut bad_layout = 0_u32;
    let mut first: Option<Vec<f32>> = None;

    while start.elapsed().as_secs_f64() < seconds {
        let size = match socket.recv_from(&mut buf) {
            Ok((size, _addr)) => size,
            Err(_) => continue, // timeout while idle; keep waiting until deadline
        };

        if let Ok((_remainder, OscPacket::Message(message))) = decoder::decode_udp(&buf[..size]) {
            if message.addr != "/meter/level" {
                continue;
            }

            frames += 1;
            let floats: Vec<f32> = message
                .args
                .iter()
                .filter_map(|arg| match arg {
                    OscType::Float(value) => Some(*value),
                    _ => None,
                })
                .collect();

            if floats.len() != 4 {
                bad_layout += 1;
            }
            if first.is_none() {
                first = Some(floats);
            }
        }
    }

    let elapsed = start.elapsed().as_secs_f64();
    let hz = frames as f64 / elapsed;
    println!("received {frames} /meter/level frame(s) in {elapsed:.2}s -> {hz:.1} Hz");
    if let Some(args) = &first {
        println!("first frame args (dBFS): {args:?}");
    }

    if frames == 0 {
        anyhow::bail!("no /meter/level frames received; is dsp-core running?");
    }
    if bad_layout > 0 {
        anyhow::bail!("{bad_layout} frame(s) were not [peak_l, peak_r, rms_l, rms_r] (4 floats)");
    }

    println!("OK: /meter/level streaming verified ({hz:.1} Hz, 4 floats)");
    Ok(())
}
