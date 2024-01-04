use crate::synths::oscillator::Oscillator;
use std::sync::mpsc::{self, Receiver, Sender};

pub mod channel;

pub enum Message {
    Synth(Oscillator),
    StreamStart,
    StreamStop,
}

pub fn get_control_channel() -> (Sender<Message>, Receiver<Message>) {
    return mpsc::channel();
}
