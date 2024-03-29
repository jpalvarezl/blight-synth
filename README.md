## blight-synth

This project is a synthesizer application built with Rust. It uses the cpal library to handle audio streaming and provides a UI for user interaction.
When I have the foundation set, I would like to make this sort of a "distortion" focused synth.

## Concepts
This project is divided into two main parts: the `core` and the `ui`.

- `core` is responsible for generating and processing audio. It includes an Oscillator for generating waveforms, a Stream for streaming audio to the audio device, and various other components for handling audio processing tasks.

- `ui` is responsible for handling user input and displaying the state of the synthesizer. It uses the egui library for the graphical interface.

## Setup
To set up the project, follow these steps:

- Clone the repository.
- Navigate to the project directory.
- Run cargo build to compile the project.
- Run cargo run to start the application.

## Dependencies
This project uses the following main dependencies:

[cpal](https://github.com/RustAudio/cpal): A cross-platform audio I/O library in pure Rust.
[egui](https://github.com/emilk/egui): An easy-to-use immediate mode GUI in pure Rust.

## Details

The audio streaming is handled by the make_stream function in the devices module. This function creates an audio stream using the cpal library. It takes a reference to a `cpal::Device` and a `cpal::StreamConfig` as arguments. The `cpal::Device` represents the audio device that will be used to play the sound, and the `cpal::StreamConfig` contains the configuration for the audio stream.

The `Oscillator` is responsible for generating the audio signal. It supports different waveforms, including sine, square, saw, and triangle. The waveform can be set using the set_waveform method, and the frequency can be set using the set_frequency method. The tick method generates the next sample of the waveform.

The process_frame function in the devices module is responsible for filling the audio buffer with the audio signal generated by the `Oscillator`. It takes a mutable reference to the audio buffer, a mutable reference to the `Oscillator`, and the number of channels as arguments. It generates a sample for each frame in the buffer using the `Oscillator` and writes it to the buffer.

The audio stream is started by calling the play method on the `cpal::Stream`. This starts the audio device and begins calling the callback function to fill the audio buffer.

The UI uses the `egui` library to handle user input and display the state of the synthesizer. It includes components for setting the waveform and frequency of the `Oscillator`, and for starting and stopping the audio stream.
