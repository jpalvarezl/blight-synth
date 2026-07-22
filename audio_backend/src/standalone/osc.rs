use anyhow::{Context, Result};
use rosc::{decoder, encoder, OscMessage, OscPacket, OscType};
use std::net::{SocketAddr, ToSocketAddrs};
use std::path::PathBuf;
use std::sync::mpsc::TrySendError;
use std::time::Duration;
use tokio::net::UdpSocket;

use crate::{id::EffectId, AudioBackendError, MeterLevels, MeterState, MixerCmd, TransportCmd};

use super::control_worker::{OscCommandRequest, StandaloneControlWorker};

pub const OSC_LISTEN_ADDR: &str = "127.0.0.1:9000";
pub const OSC_SEND_ADDR: &str = "127.0.0.1:9001";

/// Reserved master gain effect used by the standalone OSC bridge.
///
/// `/param/set gain <0..1>` carries a *normalized* control value (linear
/// amplitude, the VST/AU parameter convention). The core maps it to the dB
/// the master `Gain` effect expects via [`normalized_gain_to_db`] and emits
/// `MixerCmd::SetMasterEffectParameter`. [`StandaloneControlWorker::spawn`]
/// installs this effect while initializing `BlightAudio` on its worker thread.
pub const MASTER_GAIN_EFFECT_ID: EffectId = 0;
pub const MASTER_GAIN_PARAM_INDEX: u32 = 0;

/// Target meter streaming rate (`/meter/level`) in Hz.
pub const METER_RATE_HZ: u32 = 30;
/// Interval between `/meter/level` messages, derived from [`METER_RATE_HZ`].
const METER_INTERVAL: Duration = Duration::from_micros(1_000_000 / METER_RATE_HZ as u64);
/// Level reported for silence / non-finite values, in dBFS.
const METER_FLOOR_DB: f32 = -120.0;
/// dB floor for the normalized master gain mapping (effectively mute).
const GAIN_FLOOR_DB: f32 = -120.0;
/// Poll cadence for protocol responses completed by the NRT control worker.
const CONTROL_RESPONSE_INTERVAL: Duration = Duration::from_millis(1);

pub struct OscServer {
    socket: UdpSocket,
    send_addr: SocketAddr,
}

#[derive(Default)]
struct OscDispatch {
    commands: Vec<OscCommandRequest>,
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

    /// Runs the OSC receive loop alongside `/meter/level` streaming at
    /// [`METER_RATE_HZ`]. Incoming packets are handed to the dedicated NRT
    /// control worker; this Tokio task never blocks on RT queue saturation.
    pub async fn run_with_meter(
        &self,
        control: &mut StandaloneControlWorker,
        meter: &MeterState,
    ) -> Result<()> {
        let mut buf = [0_u8; decoder::MTU];
        let mut meter_timer = tokio::time::interval(METER_INTERVAL);
        meter_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut response_timer = tokio::time::interval(CONTROL_RESPONSE_INTERVAL);
        response_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                recv = self.socket.recv_from(&mut buf) => {
                    let (size, _remote_addr) =
                        recv.context("failed to receive OSC UDP packet")?;

                    let packet = match decoder::decode_udp(&buf[..size]) {
                        Ok((_remainder, packet)) => packet,
                        Err(err) => {
                            log::warn!("dropping invalid OSC packet: {err}");
                            continue;
                        }
                    };

                    for response in self.enqueue_dispatch(control, dispatch_packet(packet)) {
                        self.send_packet(&response).await?;
                    }
                }
                _ = meter_timer.tick() => {
                    let levels = meter.take_levels();
                    self.send_packet(&meter_level(&levels)).await?;
                }
                _ = response_timer.tick() => {
                    if !control.is_running() {
                        return Err(AudioBackendError(
                            "standalone control worker stopped".to_string(),
                        ).into());
                    }
                    for response in control.drain_responses() {
                        self.send_packet(&response).await?;
                    }
                }
            }
        }
    }

    /// Hands decoded work to the NRT control worker and returns responses that
    /// do not depend on audio-command acceptance.
    fn enqueue_dispatch(
        &self,
        control: &StandaloneControlWorker,
        dispatch: OscDispatch,
    ) -> Vec<OscPacket> {
        let mut responses = dispatch.responses;

        for path in dispatch.song_loads {
            match control.try_load_song(path) {
                Ok(()) => {}
                Err(TrySendError::Full(path)) => {
                    log::warn!("standalone control worker request queue is full");
                    responses.push(song_load_error(
                        &path,
                        "standalone control worker request queue is full",
                    ));
                }
                Err(TrySendError::Disconnected(path)) => {
                    log::error!("standalone control worker is disconnected");
                    responses.push(song_load_error(
                        &path,
                        "standalone control worker is disconnected",
                    ));
                }
            }
        }

        if !dispatch.commands.is_empty() {
            match control.try_submit_commands(dispatch.commands) {
                Ok(()) => {}
                Err(TrySendError::Full(_submissions)) => {
                    log::warn!(
                        "dropping unaccepted OSC command batch: control worker queue is full"
                    );
                }
                Err(TrySendError::Disconnected(_submissions)) => {
                    log::error!(
                        "dropping unaccepted OSC command batch: control worker is disconnected"
                    );
                }
            }
        }

        responses
    }

    async fn send_packet(&self, packet: &OscPacket) -> Result<()> {
        log::trace!("sending OSC packet: {packet:?}");
        let encoded = encoder::encode(packet).context("failed to encode OSC packet")?;
        self.socket
            .send_to(&encoded, self.send_addr)
            .await
            .with_context(|| format!("failed to send OSC packet to {}", self.send_addr))?;
        Ok(())
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
                commands: vec![OscCommandRequest {
                    command: TransportCmd::PlayLastSong.into(),
                    accepted_response: None,
                }],
                song_loads: Vec::new(),
                responses: Vec::new(),
            }
        }
        "/transport/stop" => {
            log::info!("OSC /transport/stop -> TransportCmd::StopSong");
            OscDispatch {
                commands: vec![OscCommandRequest {
                    command: TransportCmd::StopSong.into(),
                    accepted_response: None,
                }],
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
        // Wire format is a normalized 0..1 control value (linear amplitude);
        // map it to the dB the master Gain effect expects.
        "gain" => {
            let normalized = value.clamp(0.0, 1.0);
            let db = normalized_gain_to_db(normalized);
            log::info!(
                "OSC /param/set gain {normalized} (norm) -> {db} dB -> MixerCmd::SetMasterEffectParameter"
            );
            OscDispatch {
                commands: vec![OscCommandRequest {
                    command: MixerCmd::SetMasterEffectParameter {
                        effect_id: MASTER_GAIN_EFFECT_ID,
                        param_index: MASTER_GAIN_PARAM_INDEX,
                        value: db,
                    }
                    .into(),
                    // Echo only after the bounded audio queue accepts the value.
                    accepted_response: Some(param_echo(param_id, normalized)),
                }],
                song_loads: Vec::new(),
                responses: Vec::new(),
            }
        }
        unknown => {
            log::warn!("unknown parameter id: {unknown}");
            OscDispatch::default()
        }
    }
}

/// Maps a normalized gain control value (`0..1`, linear amplitude) to dB for
/// the master `Gain` effect. `1.0` is unity (`0 dB`), `0.0` (and any value at
/// or below it) is silence, floored at [`GAIN_FLOOR_DB`]. Values are clamped to
/// `0..1` by the caller.
fn normalized_gain_to_db(value: f32) -> f32 {
    if value <= 0.0 {
        return GAIN_FLOOR_DB;
    }
    (20.0 * value.log10()).max(GAIN_FLOOR_DB)
}

fn param_echo(param_id: &str, value: f32) -> OscPacket {
    OscPacket::Message(OscMessage {
        addr: "/param/echo".to_string(),
        args: vec![OscType::String(param_id.to_string()), OscType::Float(value)],
    })
}

pub(super) fn song_loaded(path: &std::path::Path, song_name: &str) -> OscPacket {
    OscPacket::Message(OscMessage {
        addr: "/song/loaded".to_string(),
        args: vec![
            OscType::String(path.display().to_string()),
            OscType::String(song_name.to_string()),
        ],
    })
}

pub(super) fn song_load_error(path: &std::path::Path, error: &str) -> OscPacket {
    OscPacket::Message(OscMessage {
        addr: "/song/error".to_string(),
        args: vec![
            OscType::String(path.display().to_string()),
            OscType::String(error.to_string()),
        ],
    })
}

/// Converts a non-negative linear amplitude to dBFS, flooring silence and
/// non-finite values at [`METER_FLOOR_DB`].
fn amp_to_db(amp: f32) -> f32 {
    if !amp.is_finite() || amp <= 0.0 {
        return METER_FLOOR_DB;
    }
    (20.0 * amp.log10()).max(METER_FLOOR_DB)
}

/// Builds a `/meter/level` message: `[peak_l, peak_r, rms_l, rms_r]` in dBFS.
fn meter_level(levels: &MeterLevels) -> OscPacket {
    OscPacket::Message(OscMessage {
        addr: "/meter/level".to_string(),
        args: vec![
            OscType::Float(amp_to_db(levels.peak_left)),
            OscType::Float(amp_to_db(levels.peak_right)),
            OscType::Float(amp_to_db(levels.rms_left)),
            OscType::Float(amp_to_db(levels.rms_right)),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Command;

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

    fn accepted_response(command: OscCommandRequest) -> OscPacket {
        command
            .accepted_response
            .expect("expected an acceptance response")
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
    fn param_set_gain_maps_normalized_value_to_db_and_echoes_normalized() {
        let dispatch = dispatch_packet(message(
            "/param/set",
            vec![OscType::String("gain".to_string()), OscType::Float(0.5)],
        ));

        assert_eq!(dispatch.commands.len(), 1);
        let Command::Mixer(MixerCmd::SetMasterEffectParameter {
            effect_id,
            param_index,
            value,
        }) = &dispatch.commands[0].command
        else {
            panic!("expected MixerCmd::SetMasterEffectParameter");
        };
        assert_eq!(*effect_id, MASTER_GAIN_EFFECT_ID);
        assert_eq!(*param_index, MASTER_GAIN_PARAM_INDEX);
        // 0.5 linear amplitude ~= -6.02 dB.
        assert!((*value - (-6.0206)).abs() < 1e-3, "got {value}");

        // Echo is held until queue submission confirms acceptance.
        assert!(dispatch.responses.is_empty());
        let response = accepted_response(dispatch.commands.into_iter().next().unwrap());
        let (id, echoed) = param_echo_args(&response);
        assert_eq!(id, "gain");
        assert!((echoed - 0.5).abs() < 1e-6);
    }

    #[test]
    fn param_set_gain_accepts_int_values() {
        // Int 1 -> unity gain (0 dB), echoed as 1.0.
        let dispatch = dispatch_packet(message(
            "/param/set",
            vec![OscType::String("gain".to_string()), OscType::Int(1)],
        ));

        let Command::Mixer(MixerCmd::SetMasterEffectParameter { value, .. }) =
            &dispatch.commands[0].command
        else {
            panic!("expected MixerCmd::SetMasterEffectParameter");
        };
        assert!(
            (*value - 0.0).abs() < 1e-4,
            "unity gain -> 0 dB, got {value}"
        );
        let response = accepted_response(dispatch.commands.into_iter().next().unwrap());
        assert!((param_echo_args(&response).1 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn param_set_gain_clamps_to_unit_range() {
        // Above 1.0 clamps to unity (0 dB).
        let high = dispatch_packet(message(
            "/param/set",
            vec![OscType::String("gain".to_string()), OscType::Float(2.0)],
        ));
        let Command::Mixer(MixerCmd::SetMasterEffectParameter { value, .. }) =
            &high.commands[0].command
        else {
            panic!("expected MixerCmd::SetMasterEffectParameter");
        };
        assert!((*value - 0.0).abs() < 1e-4);
        let response = accepted_response(high.commands.into_iter().next().unwrap());
        assert!((param_echo_args(&response).1 - 1.0).abs() < 1e-6);

        // Zero (and below) floors to silence.
        let low = dispatch_packet(message(
            "/param/set",
            vec![OscType::String("gain".to_string()), OscType::Float(0.0)],
        ));
        let Command::Mixer(MixerCmd::SetMasterEffectParameter { value, .. }) =
            &low.commands[0].command
        else {
            panic!("expected MixerCmd::SetMasterEffectParameter");
        };
        assert_eq!(*value, GAIN_FLOOR_DB);
        let response = accepted_response(low.commands.into_iter().next().unwrap());
        assert!((param_echo_args(&response).1 - 0.0).abs() < 1e-6);
    }

    #[test]
    fn transport_play_and_stop_translate_to_existing_commands() {
        let play_dispatch = dispatch_packet(message("/transport/play", vec![]));
        assert_eq!(play_dispatch.commands.len(), 1);
        assert!(matches!(
            play_dispatch.commands[0].command,
            Command::Transport(TransportCmd::PlayLastSong)
        ));

        let stop_dispatch = dispatch_packet(message("/transport/stop", vec![]));
        assert_eq!(stop_dispatch.commands.len(), 1);
        assert!(matches!(
            stop_dispatch.commands[0].command,
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

    #[test]
    fn amp_to_db_floors_silence_and_non_finite() {
        assert_eq!(amp_to_db(0.0), METER_FLOOR_DB);
        assert_eq!(amp_to_db(-1.0), METER_FLOOR_DB);
        assert_eq!(amp_to_db(f32::NAN), METER_FLOOR_DB);
        assert_eq!(amp_to_db(f32::INFINITY), METER_FLOOR_DB);
    }

    #[test]
    fn amp_to_db_maps_known_amplitudes() {
        assert!((amp_to_db(1.0) - 0.0).abs() < 1e-4);
        assert!((amp_to_db(0.5) - (-6.0206)).abs() < 1e-3);
    }

    #[test]
    fn meter_level_emits_four_db_floats() {
        let packet = meter_level(&MeterLevels {
            peak_left: 1.0,
            peak_right: 0.5,
            rms_left: 1.0,
            rms_right: 0.0,
        });

        let OscPacket::Message(message) = packet else {
            panic!("expected OSC message");
        };
        assert_eq!(message.addr, "/meter/level");
        let [OscType::Float(peak_l), OscType::Float(peak_r), OscType::Float(rms_l), OscType::Float(rms_r)] =
            message.args.as_slice()
        else {
            panic!("expected /meter/level args [f32; 4]");
        };
        assert!((peak_l - 0.0).abs() < 1e-4);
        assert!((peak_r - (-6.0206)).abs() < 1e-3);
        assert!((rms_l - 0.0).abs() < 1e-4);
        assert_eq!(*rms_r, METER_FLOOR_DB);
    }
}
