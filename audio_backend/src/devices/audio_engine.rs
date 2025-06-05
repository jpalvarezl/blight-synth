use crossbeam::queue::SegQueue;
use std::sync::Arc;

use crate::synths::waveform::Waveform;
// use super::commands::SynthesizerCommand;

#[derive(Debug, Clone)]
pub enum AudioCommand {
    // Synthesizer control commands
    NoteOn {
        note: u8,
        velocity: f32,
    },
    NoteOnWithWaveform {
        note: u8,
        velocity: f32,
        waveform: Waveform,
    },
    NoteOff {
        note: u8,
    },
    SetAdsr {
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    },
    SetWaveform(Waveform),
    // Engine control
    // Stop,
}

pub struct AudioEngine {
    command_queue: Arc<SegQueue<AudioCommand>>,
}

impl AudioEngine {
    pub fn new() -> anyhow::Result<Self> {
        // Create the lock-free command queue
        let command_queue = Arc::new(SegQueue::new());
        // let command_queue_audio = Arc::clone(&command_queue);

        // // Create the stream with command processing
        // let stream = super::streams::create_stream_with_commands(
        //     sample_rate,
        //     max_polyphony,
        //     command_queue_audio
        // )?;

        // // Start the stream
        // stream.play()?;

        Ok(Self { command_queue })
    }

    pub fn command_queue(&self) -> Arc<SegQueue<AudioCommand>> {
        Arc::clone(&self.command_queue)
    }

    // High-level API for GUI thread
    pub fn note_on(&self, note: u8, velocity: f32) {
        self.command_queue
            .push(AudioCommand::NoteOn { note, velocity });
    }

    pub fn note_on_with_waveform(&self, note: u8, velocity: f32, waveform: Waveform) {
        self.command_queue.push(AudioCommand::NoteOnWithWaveform {
            note,
            velocity,
            waveform,
        });
    }

    pub fn note_off(&self, note: u8) {
        self.command_queue.push(AudioCommand::NoteOff { note });
    }

    pub fn set_attack(&self, attack: f32) {
        if let Some((_, decay, sustain, release)) = self.get_current_adsr() {
            self.command_queue.push(AudioCommand::SetAdsr {
                attack,
                decay,
                sustain,
                release,
            });
        }
    }

    pub fn set_decay(&self, decay: f32) {
        if let Some((attack, _, sustain, release)) = self.get_current_adsr() {
            self.command_queue.push(AudioCommand::SetAdsr {
                attack,
                decay,
                sustain,
                release,
            });
        }
    }

    pub fn set_sustain(&self, sustain: f32) {
        if let Some((attack, decay, _, release)) = self.get_current_adsr() {
            self.command_queue.push(AudioCommand::SetAdsr {
                attack,
                decay,
                sustain,
                release,
            });
        }
    }

    pub fn set_release(&self, release: f32) {
        if let Some((attack, decay, sustain, _)) = self.get_current_adsr() {
            self.command_queue.push(AudioCommand::SetAdsr {
                attack,
                decay,
                sustain,
                release,
            });
        }
    }

    pub fn set_adsr(&self, attack: f32, decay: f32, sustain: f32, release: f32) {
        self.command_queue.push(AudioCommand::SetAdsr {
            attack,
            decay,
            sustain,
            release,
        });
    }

    pub fn set_waveform(&self, waveform: Waveform) {
        self.command_queue.push(AudioCommand::SetWaveform(waveform));
    }

    // pub fn stop(&self) {
    //     self.command_queue.push(AudioCommand::Stop);
    // }

    // Helper method - you might want to implement this differently
    // depending on how you want to handle parameter queries
    fn get_current_adsr(&self) -> Option<(f32, f32, f32, f32)> {
        // This is a simplified approach - in practice you might want
        // to maintain a separate copy of parameters for queries
        Some((0.01, 0.1, 0.8, 0.2)) // Default values
    }
}
