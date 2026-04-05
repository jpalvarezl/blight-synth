use crate::state::AppState;
use anyhow::Result;
use rosc::{OscMessage, OscPacket};
use std::net::UdpSocket;

const OSC_LISTEN_ADDR: &str = "127.0.0.1:9000";
const OSC_SEND_ADDR: &str = "127.0.0.1:9001"; // GUI listens here

pub struct OscServer {
    state: AppState,
}

impl OscServer {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    /// Bind UDP socket and dispatch incoming OSC messages.
    pub async fn run(&self) -> Result<()> {
        // TODO: bind UdpSocket to OSC_LISTEN_ADDR
        // TODO: loop: recv_from -> decode OscPacket -> dispatch
        Ok(())
    }

    /// Route a decoded OSC message to the appropriate handler.
    fn dispatch(&self, msg: OscMessage) {
        match msg.addr.as_str() {
            "/param/set" => self.handle_param_set(msg),
            "/transport/play" => self.handle_transport_play(msg),
            "/transport/stop" => self.handle_transport_stop(msg),
            "/preset/load" => self.handle_preset_load(msg),
            _ => log::warn!("Unknown OSC address: {}", msg.addr),
        }
    }

    fn handle_param_set(&self, msg: OscMessage) {
        // TODO: extract param id + value from msg.args
        // TODO: update state atomically
    }

    fn handle_transport_play(&self, msg: OscMessage) {
        // TODO: signal engine to start playback
    }

    fn handle_transport_stop(&self, msg: OscMessage) {
        // TODO: signal engine to stop playback
    }

    fn handle_preset_load(&self, msg: OscMessage) {
        // TODO: deserialize preset from msg.args
        // TODO: apply preset to state
    }

    /// Push a metering/state update to the GUI.
    pub fn send_meter_update(&self, level_db: f32) {
        // TODO: encode OscMessage { addr: "/meter/level", args: [level_db] }
        // TODO: send to OSC_SEND_ADDR
    }

    pub fn send_param_echo(&self, param_id: &str, value: f32) {
        // TODO: encode and send /param/echo back to GUI for confirmation
    }
}
