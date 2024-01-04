use std::sync::mpsc::{self, Sender, Receiver};
use control::{Message, get_control_channel};
use cpal::traits::{DeviceTrait, StreamTrait};
use synths::oscilator::{self, Oscillator};

pub mod harmony;
pub mod synths;
pub mod devices;
pub mod control;

type Result<T> = anyhow::Result<T, anyhow::Error>;

pub fn play_the_thing() -> Result<Sender<Message>> {
    let (control_sender, control_receiver) = get_control_channel();
    let _thread_handle = std::thread::spawn(move||{
        // let mut oscillator = Oscillator {
        //     waveform: oscilator::Waveform::Sine,
        //     sample_rate: 44100.0,
        //     current_sample_index: 0.0,
        //     frequency_hz: 440.0,
        // };
        // let message = control_receiver.recv().expect("Control channel error");
        let stream = devices::setup_stream().expect("Stream setup error");

        stream.play().expect("Stream start error");
        std::thread::sleep(std::time::Duration::from_secs(5));
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
