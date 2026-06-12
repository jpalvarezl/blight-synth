use anyhow::{Context, Result};
use rosc::{decoder, encoder, OscMessage, OscPacket, OscType};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use tokio::net::UdpSocket;

use crate::{
    id::EffectId, load_song_file_into_audio, BlightAudio, Command, MixerCmd, TransportCmd,
};

pub const OSC_LISTEN_ADDR: &str = "127.0.0.1:9000";
pub const OSC_SEND_ADDR: &str = "127.0.0.1:9001";

/// Reserved master gain effect used by the standalone OSC bridge.
///
/// `/param/set gain <db>` translates to `MixerCmd::SetMasterEffectParameter`
/// for this effect. The standalone binary is responsible for installing this
/// effect during startup.
pub const MASTER_GAIN_EFFECT_ID: EffectId = 0;
pub const MASTER_GAIN_PARAM_INDEX: u32 = 0;

pub struct OscServer {
    socket: UdpSocket,
    send_addr: SocketAddr,
}

#[derive(Default)]
struct OscDispatch {
    commands: Vec<Command>,
    song_loads: Vec<PathBuf>,
    responses: Vec<OscPacket>,
}

impl OscDispatch {
    fn append(&mut self, mut other: Self) {
        self.commands.append(&mut other.commands);
        self.song_loads.append(&mut other.song_loads);
        self.responses.append(&mut other.responses);
    }
}

impl OscServer {
    pub async fn bind() -> Result<Self> {
        Self::bind_to(OSC_LISTEN_ADDR, OSC_SEND_ADDR).await
    }

    pub async fn bind_to(
        listen_addr: impl Into<String>,
        send_addr: impl Into<String>,
    ) -> Result<Self> {
        let listen_addr = listen_addr.into();
        let socket = UdpSocket::bind(&listen_addr)
            .await
            .with_context(|| format!("failed to bind OSC UDP socket on {listen_addr}"))?;

        // Resolve the response address once at bind time so the hot send loop
        // does not re-parse/resolve a string on every outgoing packet.
        let send_addr_str = send_addr.into();
        let send_addr = send_addr_str
            .to_socket_addrs()
            .with_context(|| format!("failed to resolve OSC send address {send_addr_str}"))?
            .next()
            .with_context(|| format!("no socket address resolved for {send_addr_str}"))?;
        log::info!("OSC server listening on {listen_addr}; responses -> {send_addr}");

        Ok(Self { socket, send_addr })
    }

    pub async fn run(&self, audio: &mut BlightAudio) -> Result<()> {
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

            let mut dispatch = dispatch_packet(packet);

            for path in dispatch.song_loads {
                match load_song_file_into_audio(audio, &path) {
                    Ok(song) => {
                        log::info!("loaded song '{}' from {}", song.name, path.display());
                        dispatch.responses.push(song_loaded(&path, &song.name));
                    }
                    Err(err) => {
                        log::error!("failed to load song from {}: {err:?}", path.display());
                        dispatch
                            .responses
                            .push(song_load_error(&path, &err.to_string()));
                    }
                }
            }

            for command in dispatch.commands {
                log::debug!("dispatching OSC-derived command");
                audio.send_command(command);
            }

            for response in dispatch.responses {
                log::debug!("sending OSC response: {response:?}");
                let encoded =
                    encoder::encode(&response).context("failed to encode OSC response")?;
                self.socket
                    .send_to(&encoded, self.send_addr)
                    .await
                    .with_context(|| {
                        format!("failed to send OSC response to {}", self.send_addr)
                    })?;
            }
        }
    }
}

fn dispatch_packet(packet: OscPacket) -> OscDispatch {
    match packet {
        OscPacket::Message(message) => handle_message(message),
        OscPacket::Bundle(bundle) => {
            let mut dispatch = OscDispatch::default();
            for packet in bundle.content {
                dispatch.append(dispatch_packet(packet));
            }
            dispatch
        }
    }
}

fn handle_message(message: OscMessage) -> OscDispatch {
    match message.addr.as_str() {
        "/param/set" => handle_param_set(message),
        "/song/load" => handle_song_load(message),
        "/transport/play" => {
            log::info!("OSC /transport/play -> TransportCmd::PlayLastSong");
            OscDispatch {
                commands: vec![TransportCmd::PlayLastSong.into()],
                song_loads: Vec::new(),
                responses: Vec::new(),
            }
        }
        "/transport/stop" => {
            log::info!("OSC /transport/stop -> TransportCmd::StopSong");
            OscDispatch {
                commands: vec![TransportCmd::StopSong.into()],
                song_loads: Vec::new(),
                responses: Vec::new(),
            }
        }
        unknown => {
            log::warn!("unknown OSC address: {unknown}");
            OscDispatch::default()
        }
    }
}

fn handle_song_load(message: OscMessage) -> OscDispatch {
    let [OscType::String(path)] = message.args.as_slice() else {
        log::warn!("invalid /song/load args; expected [string path]");
        return OscDispatch::default();
    };

    log::info!("OSC /song/load {path}");
    OscDispatch {
        commands: Vec::new(),
        song_loads: vec![PathBuf::from(path)],
        responses: Vec::new(),
    }
}

fn handle_param_set(message: OscMessage) -> OscDispatch {
    let [OscType::String(param_id), value] = message.args.as_slice() else {
        log::warn!("invalid /param/set args; expected [string, float or int]");
        return OscDispatch::default();
    };

    let value = match value {
        OscType::Float(value) => *value,
        OscType::Int(value) => *value as f32,
        _ => {
            log::warn!("invalid /param/set value for {param_id}; expected float or int");
            return OscDispatch::default();
        }
    };

    match param_id.as_str() {
        // Existing Gain::set_parameter semantics are dB, so this OSC value is dB.
        "gain" => {
            log::info!("OSC /param/set gain {value} dB -> MixerCmd::SetMasterEffectParameter");
            OscDispatch {
                commands: vec![MixerCmd::SetMasterEffectParameter {
                    effect_id: MASTER_GAIN_EFFECT_ID,
                    param_index: MASTER_GAIN_PARAM_INDEX,
                    value,
                }
                .into()],
                song_loads: Vec::new(),
                responses: vec![param_echo(param_id, value)],
            }
        }
        unknown => {
            log::warn!("unknown parameter id: {unknown}");
            OscDispatch::default()
        }
    }
}

fn param_echo(param_id: &str, value: f32) -> OscPacket {
    OscPacket::Message(OscMessage {
        addr: "/param/echo".to_string(),
        args: vec![OscType::String(param_id.to_string()), OscType::Float(value)],
    })
}

fn song_loaded(path: &std::path::Path, song_name: &str) -> OscPacket {
    OscPacket::Message(OscMessage {
        addr: "/song/loaded".to_string(),
        args: vec![
            OscType::String(path.display().to_string()),
            OscType::String(song_name.to_string()),
        ],
    })
}

fn song_load_error(path: &std::path::Path, error: &str) -> OscPacket {
    OscPacket::Message(OscMessage {
        addr: "/song/error".to_string(),
        args: vec![
            OscType::String(path.display().to_string()),
            OscType::String(error.to_string()),
        ],
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
    fn song_load_records_path_for_runtime_loading() {
        let dispatch = dispatch_packet(message(
            "/song/load",
            vec![OscType::String("calibration.json".to_string())],
        ));

        assert!(dispatch.commands.is_empty());
        assert_eq!(dispatch.song_loads, vec![PathBuf::from("calibration.json")]);
        assert!(dispatch.responses.is_empty());
    }

    #[test]
    fn invalid_song_load_does_not_emit_action() {
        let dispatch = dispatch_packet(message("/song/load", vec![OscType::Float(1.0)]));

        assert!(dispatch.commands.is_empty());
        assert!(dispatch.song_loads.is_empty());
        assert!(dispatch.responses.is_empty());
    }

    #[test]
    fn param_set_gain_translates_to_existing_master_gain_command_and_echo() {
        let dispatch = dispatch_packet(message(
            "/param/set",
            vec![OscType::String("gain".to_string()), OscType::Float(-6.0)],
        ));

        assert_eq!(dispatch.commands.len(), 1);
        let Command::Mixer(MixerCmd::SetMasterEffectParameter {
            effect_id,
            param_index,
            value,
        }) = &dispatch.commands[0]
        else {
            panic!("expected MixerCmd::SetMasterEffectParameter");
        };
        assert_eq!(*effect_id, MASTER_GAIN_EFFECT_ID);
        assert_eq!(*param_index, MASTER_GAIN_PARAM_INDEX);
        assert_eq!(*value, -6.0);

        assert_eq!(dispatch.responses.len(), 1);
        assert_eq!(param_echo_args(&dispatch.responses[0]), ("gain", -6.0));
    }

    #[test]
    fn param_set_gain_accepts_int_values() {
        let dispatch = dispatch_packet(message(
            "/param/set",
            vec![OscType::String("gain".to_string()), OscType::Int(-12)],
        ));

        let Command::Mixer(MixerCmd::SetMasterEffectParameter { value, .. }) =
            &dispatch.commands[0]
        else {
            panic!("expected MixerCmd::SetMasterEffectParameter");
        };
        assert_eq!(*value, -12.0);
        assert_eq!(param_echo_args(&dispatch.responses[0]), ("gain", -12.0));
    }

    #[test]
    fn transport_play_and_stop_translate_to_existing_commands() {
        let play_dispatch = dispatch_packet(message("/transport/play", vec![]));
        assert_eq!(play_dispatch.commands.len(), 1);
        assert!(matches!(
            play_dispatch.commands[0],
            Command::Transport(TransportCmd::PlayLastSong)
        ));

        let stop_dispatch = dispatch_packet(message("/transport/stop", vec![]));
        assert_eq!(stop_dispatch.commands.len(), 1);
        assert!(matches!(
            stop_dispatch.commands[0],
            Command::Transport(TransportCmd::StopSong)
        ));
    }

    #[test]
    fn invalid_param_set_does_not_emit_command_or_echo() {
        let dispatch = dispatch_packet(message(
            "/param/set",
            vec![
                OscType::String("gain".to_string()),
                OscType::String("bad".to_string()),
            ],
        ));

        assert!(dispatch.commands.is_empty());
        assert!(dispatch.responses.is_empty());
    }

    #[test]
    fn unknown_address_is_ignored() {
        let dispatch = dispatch_packet(message("/unknown", vec![]));

        assert!(dispatch.commands.is_empty());
        assert!(dispatch.responses.is_empty());
    }
}
