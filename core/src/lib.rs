use std::{sync::{mpsc::{self, Sender, Receiver}, Arc}, ops::Deref};
use control::{Message, get_control_channel};
use cpal::traits::{DeviceTrait, StreamTrait};
use synths::oscillator::{self, Oscillator};

pub mod harmony;
pub mod synths;
pub mod devices;
pub mod control;

type Result<T> = anyhow::Result<T, anyhow::Error>;

pub fn play_the_thing(oscillator: Arc<Oscillator>) -> Result<Sender<Message>> {
    // The oscillator is a shared resource between the audio thread and the UI thread
    // Therefore should be received in the channel

    let (control_sender, control_receiver) = get_control_channel();
    let _thread_handle = std::thread::spawn(move||{
        let stream = devices::setup_stream(oscillator).expect("Stream setup error");

        stream.play().expect("Stream start error");
        std::thread::sleep(std::time::Duration::from_millis(1050));
        stream.pause().expect("Stream stop error");

        // for message in control_receiver {
        //     match message {
        //         Message::Synth(oscillator) => {
        //             println!("Synth message received");
        //             // oscillator = oscillator;
        //         },
        //         Message::StreamStart => {
        //             println!("Stream start message received");
        //             stream.play().expect("Stream start error");
        //         },
        //         Message::StreamStop => {
        //             println!("Stream stop message received");
        //             stream.pause().expect("Stream stop error");
        //         },
        //     }
        // }

    });
    
    Ok(control_sender)
}

pub fn get_default_output_device_name() -> Result<String> {
    let default_device = devices::get_default_device()?;
    Ok(default_device.name()?)
}
