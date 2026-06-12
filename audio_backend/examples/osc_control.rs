use std::{net::UdpSocket, time::Duration};

use anyhow::{Context, Result};
use rosc::{decoder, encoder, OscMessage, OscPacket, OscType};

const DSP_OSC_ADDR: &str = "127.0.0.1:9000";
const GUI_OSC_ADDR: &str = "127.0.0.1:9001";

fn main() -> Result<()> {
    env_logger::init();

    println!("This example sends OSC to a running dsp-core process.");
    println!("Start it first with: cargo run -p audio_backend --bin dsp-core");

    let socket =
        UdpSocket::bind(GUI_OSC_ADDR).context("failed to bind local OSC receive socket")?;
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .context("failed to configure OSC receive timeout")?;

    send_message(
        &socket,
        "/song/load",
        vec![OscType::String("calibration.json".to_string())],
    )?;
    println!("sent /song/load calibration.json");
    recv_one(&socket, "song load response");

    send_message(
        &socket,
        "/param/set",
        vec![OscType::String("gain".to_string()), OscType::Float(-6.0)],
    )?;
    println!("sent /param/set gain -6.0 dB");
    recv_one(&socket, "param echo");

    send_message(&socket, "/transport/play", vec![])?;
    println!("sent /transport/play");

    // While playing, the dsp-core streams /meter/level at ~30 Hz to this socket.
    print_meter_levels(&socket, Duration::from_secs(2));

    send_message(&socket, "/transport/stop", vec![])?;
    println!("sent /transport/stop");

    Ok(())
}

/// Reads and prints incoming `/meter/level` messages for `duration`.
fn print_meter_levels(socket: &UdpSocket, duration: Duration) {
    let deadline = std::time::Instant::now() + duration;
    let mut buf = [0_u8; decoder::MTU];
    let mut count = 0_u32;

    while std::time::Instant::now() < deadline {
        match socket.recv_from(&mut buf) {
            Ok((size, _addr)) => {
                if let Ok((_remainder, OscPacket::Message(message))) =
                    decoder::decode_udp(&buf[..size])
                {
                    if message.addr == "/meter/level" {
                        count += 1;
                        // Throttle printing so the example stays readable.
                        if count % 10 == 1 {
                            println!("received /meter/level: {:?}", message.args);
                        }
                    }
                }
            }
            Err(_) => break, // timeout while idle
        }
    }

    println!("received {count} /meter/level message(s) during playback");
}

fn send_message(socket: &UdpSocket, addr: &str, args: Vec<OscType>) -> Result<()> {
    let packet = OscPacket::Message(OscMessage {
        addr: addr.to_string(),
        args,
    });
    let encoded = encoder::encode(&packet).context("failed to encode OSC message")?;
    socket
        .send_to(&encoded, DSP_OSC_ADDR)
        .with_context(|| format!("failed to send OSC message to {DSP_OSC_ADDR}"))?;
    Ok(())
}

fn recv_one(socket: &UdpSocket, label: &str) {
    let mut buf = [0_u8; decoder::MTU];
    match socket.recv_from(&mut buf) {
        Ok((size, _addr)) => match decoder::decode_udp(&buf[..size]) {
            Ok((_remainder, packet)) => println!("received {label}: {packet:?}"),
            Err(err) => eprintln!("received invalid {label}: {err}"),
        },
        Err(err) => eprintln!("no {label} received within timeout: {err}"),
    }
}
