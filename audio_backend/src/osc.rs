use anyhow::{Context, Result};
use rosc::{decoder, encoder, OscMessage, OscPacket, OscType};
use tokio::net::UdpSocket;

use crate::SharedAudioState;

pub const OSC_LISTEN_ADDR: &str = "127.0.0.1:9000";
pub const OSC_SEND_ADDR: &str = "127.0.0.1:9001";

pub struct OscServer {
    socket: UdpSocket,
    send_addr: String,
    state: SharedAudioState,
}

impl OscServer {
    pub async fn bind(state: SharedAudioState) -> Result<Self> {
        Self::bind_to(state, OSC_LISTEN_ADDR, OSC_SEND_ADDR).await
    }

    pub async fn bind_to(
        state: SharedAudioState,
        listen_addr: impl Into<String>,
        send_addr: impl Into<String>,
    ) -> Result<Self> {
        let listen_addr = listen_addr.into();
        let socket = UdpSocket::bind(&listen_addr)
            .await
            .with_context(|| format!("failed to bind OSC UDP socket on {listen_addr}"))?;

        Ok(Self {
            socket,
            send_addr: send_addr.into(),
            state,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let mut buf = [0_u8; decoder::MTU];

        loop {
            let (size, _remote_addr) = self
                .socket
                .recv_from(&mut buf)
                .await
                .context("failed to receive OSC UDP packet")?;

            let packet = match decoder::decode_udp(&buf[..size]) {
                Ok((_remainder, packet)) => packet,
                Err(err) => {
                    log::warn!("dropping invalid OSC packet: {err}");
                    continue;
                }
            };

            for response in dispatch_packet(&self.state, packet) {
                let encoded =
                    encoder::encode(&response).context("failed to encode OSC response")?;
                self.socket
                    .send_to(&encoded, &self.send_addr)
                    .await
                    .with_context(|| {
                        format!("failed to send OSC response to {}", self.send_addr)
                    })?;
            }
        }
    }
}

fn dispatch_packet(state: &SharedAudioState, packet: OscPacket) -> Vec<OscPacket> {
    match packet {
        OscPacket::Message(message) => handle_message(state, message),
        OscPacket::Bundle(bundle) => bundle
            .content
            .into_iter()
            .flat_map(|packet| dispatch_packet(state, packet))
            .collect(),
    }
}

fn handle_message(state: &SharedAudioState, message: OscMessage) -> Vec<OscPacket> {
    match message.addr.as_str() {
        "/param/set" => handle_param_set(state, message),
        "/transport/play" => {
            state.set_playing(true);
            Vec::new()
        }
        "/transport/stop" => {
            state.set_playing(false);
            Vec::new()
        }
        "/preset/load" => {
            log::warn!("/preset/load is not implemented yet");
            Vec::new()
        }
        unknown => {
            log::warn!("unknown OSC address: {unknown}");
            Vec::new()
        }
    }
}

fn handle_param_set(state: &SharedAudioState, message: OscMessage) -> Vec<OscPacket> {
    let [OscType::String(param_id), value] = message.args.as_slice() else {
        log::warn!("invalid /param/set args; expected [string, float]");
        return Vec::new();
    };

    let value = match value {
        OscType::Float(value) => *value,
        OscType::Int(value) => *value as f32,
        _ => {
            log::warn!("invalid /param/set value for {param_id}; expected float");
            return Vec::new();
        }
    };

    match param_id.as_str() {
        "gain" => {
            state.set_master_gain(value);
            vec![param_echo(param_id, state.master_gain())]
        }
        unknown => {
            log::warn!("unknown parameter id: {unknown}");
            Vec::new()
        }
    }
}

fn param_echo(param_id: &str, value: f32) -> OscPacket {
    OscPacket::Message(OscMessage {
        addr: "/param/echo".to_string(),
        args: vec![OscType::String(param_id.to_string()), OscType::Float(value)],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(addr: &str, args: Vec<OscType>) -> OscPacket {
        OscPacket::Message(OscMessage {
            addr: addr.to_string(),
            args,
        })
    }

    fn param_echo_args(packet: &OscPacket) -> (&str, f32) {
        let OscPacket::Message(message) = packet else {
            panic!("expected OSC message");
        };
        assert_eq!(message.addr, "/param/echo");

        let [OscType::String(param_id), OscType::Float(value)] = message.args.as_slice() else {
            panic!("expected /param/echo args [string, float]");
        };

        (param_id, *value)
    }

    #[test]
    fn param_set_gain_updates_shared_state_and_returns_echo() {
        let state = SharedAudioState::new();

        let responses = dispatch_packet(
            &state,
            message(
                "/param/set",
                vec![OscType::String("gain".to_string()), OscType::Float(0.5)],
            ),
        );

        assert_eq!(state.master_gain(), 0.5);
        assert_eq!(responses.len(), 1);
        assert_eq!(param_echo_args(&responses[0]), ("gain", 0.5));
    }

    #[test]
    fn param_set_gain_echoes_clamped_value() {
        let state = SharedAudioState::new();

        let responses = dispatch_packet(
            &state,
            message(
                "/param/set",
                vec![OscType::String("gain".to_string()), OscType::Float(2.0)],
            ),
        );

        assert_eq!(state.master_gain(), 1.0);
        assert_eq!(param_echo_args(&responses[0]), ("gain", 1.0));
    }

    #[test]
    fn transport_play_and_stop_update_shared_state() {
        let state = SharedAudioState::new();

        dispatch_packet(&state, message("/transport/play", vec![]));
        assert!(state.is_playing());

        dispatch_packet(&state, message("/transport/stop", vec![]));
        assert!(!state.is_playing());
    }

    #[test]
    fn invalid_param_set_does_not_update_state_or_emit_echo() {
        let state = SharedAudioState::new();

        let responses = dispatch_packet(
            &state,
            message(
                "/param/set",
                vec![
                    OscType::String("gain".to_string()),
                    OscType::String("bad".to_string()),
                ],
            ),
        );

        assert_eq!(state.master_gain(), 1.0);
        assert!(responses.is_empty());
    }

    #[test]
    fn unknown_address_is_ignored() {
        let state = SharedAudioState::new();

        let responses = dispatch_packet(&state, message("/unknown", vec![]));

        assert_eq!(state.master_gain(), 1.0);
        assert!(!state.is_playing());
        assert!(responses.is_empty());
    }
}
