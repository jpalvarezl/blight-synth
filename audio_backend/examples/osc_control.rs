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
        .set_read_timeout(Some(Duration::from_secs(1)))
        .context("failed to configure OSC receive timeout")?;

    send_message(
        &socket,
        "/param/set",
        vec![OscType::String("gain".to_string()), OscType::Float(-6.0)],
    )?;
    println!("sent /param/set gain -6.0 dB");

    let mut buf = [0_u8; decoder::MTU];
    match socket.recv_from(&mut buf) {
        Ok((size, _addr)) => match decoder::decode_udp(&buf[..size]) {
            Ok((_remainder, packet)) => println!("received OSC response: {packet:?}"),
            Err(err) => eprintln!("received invalid OSC response: {err}"),
        },
        Err(err) => eprintln!("no /param/echo received within timeout: {err}"),
    }

    send_message(&socket, "/transport/play", vec![])?;
    println!("sent /transport/play");

    std::thread::sleep(Duration::from_millis(500));

    send_message(&socket, "/transport/stop", vec![])?;
    println!("sent /transport/stop");

    Ok(())
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
