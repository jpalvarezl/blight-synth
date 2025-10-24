use std::sync::Arc;

use audio_backend::{InstrumentTrait};
use nih_plug::prelude::*;

pub struct HiHat {
    params: Arc<HiHatParams>,
    sample_rate: f32,
    instrument: audio_backend::HiHat,
}

#[derive(Params)]
struct HiHatParams {
    /// Attack time in seconds
    #[id = "attack"]
    pub attack: FloatParam,
    
    /// Decay time in seconds  
    #[id = "decay"]
    pub decay: FloatParam,
    
    /// Sustain level (0-1)
    #[id = "sustain"]
    pub sustain: FloatParam,
    
    /// Release time in seconds
    #[id = "release"]
    pub release: FloatParam,
    
    /// Stereo pan (-1 to 1)
    #[id = "pan"]
    pub pan: FloatParam,
}

impl Default for HiHatParams {
    fn default() -> Self {
        Self {
            attack: FloatParam::new(
                "Attack",
                0.001, // Hi-hat typically has very fast attack
                FloatRange::Skewed {
                    min: 0.0001,
                    max: 0.1,
                    factor: 0.5,
                },
            )
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(4)),
            
            decay: FloatParam::new(
                "Decay",
                0.08, // Default from your tracker_gui
                FloatRange::Skewed {
                    min: 0.001,
                    max: 1.0,
                    factor: 0.5,
                },
            )
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
            
            sustain: FloatParam::new(
                "Sustain",
                0.0, // Hi-hats typically don't sustain
                FloatRange::Linear {
                    min: 0.0,
                    max: 1.0,
                },
            )
            .with_unit("")
            .with_value_to_string(formatters::v2s_f32_percentage(2)),
            
            release: FloatParam::new(
                "Release",
                0.15, // Default from your tracker_gui
                FloatRange::Skewed {
                    min: 0.001,
                    max: 2.0,
                    factor: 0.5,
                },
            )
            .with_unit(" s")
            .with_value_to_string(formatters::v2s_f32_rounded(3)),
            
            pan: FloatParam::new(
                "Pan",
                0.0,
                FloatRange::Linear {
                    min: -1.0,
                    max: 1.0,
                },
            )
            .with_unit("")
            .with_value_to_string(formatters::v2s_f32_panning()),
        }
    }
}

impl Default for HiHat {
    fn default() -> Self {
        Self {
            params: Arc::new(HiHatParams::default()),
            sample_rate: 44100.0,
            instrument: audio_backend::HiHat::new(0, 0.0, 44100.0),
        }
    }
}

impl Plugin for HiHat {
    const NAME: &'static str = "Blight Hi-Hat";
    const VENDOR: &'static str = "Blight Synth";
    const URL: &'static str = "https://github.com/yourusername/blight-synth";
    const EMAIL: &'static str = "jp.alvarezl@gmail.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate as f32;
        self.instrument = audio_backend::HiHat::new(0, 0.0, self.sample_rate);
        true
    }

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Update parameters if they've changed
        self.update_parameters();

        // let output = buffer.as_slice();

        // Process MIDI events with sample-accurate timing
        let mut next_event = context.next_event();
        
        for (sample_id, mut channel_samples) in buffer.iter_samples().enumerate() {
            // Process any MIDI events at this sample position
            while let Some(event) = next_event {
                if event.timing() > sample_id as u32 {
                    break;
                }

                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        let velocity_u8 = (velocity * 127.0).round() as u8;
                        self.instrument.note_on(note, velocity_u8);
                    }
                    NoteEvent::NoteOff { .. } | NoteEvent::Choke { .. } => {
                        self.instrument.note_off();
                    }
                    _ => {}
                }

                next_event = context.next_event();
            }

            // Process one sample
            let mut left = [0.0f32; 1];
            let mut right = [0.0f32; 1];
            self.instrument.process(&mut left, &mut right, self.sample_rate);

            // Write to output channels
            for (ch, sample) in channel_samples.iter_mut().enumerate() {
                *sample = if ch == 0 { left[0] } else { right[0] };
            }
        }

        ProcessStatus::Normal
    }
}

impl HiHat {
    /// Update instrument parameters from the UI
    fn update_parameters(&mut self) {
        // Update envelope parameters using SynthCmd
        if self.params.attack.smoothed.is_smoothing() {
            let attack = self.params.attack.smoothed.next();
            self.instrument.try_handle_command(&audio_backend::SynthCmd::SetEnvAttack {
            envelope_id: None,
            attack,
            });
        }

        if self.params.decay.smoothed.is_smoothing() {
            let decay = self.params.decay.smoothed.next();
            self.instrument.try_handle_command(&audio_backend::SynthCmd::SetEnvDecay {
            envelope_id: None,
            decay,
            });
        }

        if self.params.sustain.smoothed.is_smoothing() {
            let sustain = self.params.sustain.smoothed.next();
            self.instrument.try_handle_command(&audio_backend::SynthCmd::SetEnvSustain {
            envelope_id: None,
            sustain,
            });
        }

        if self.params.release.smoothed.is_smoothing() {
            let release = self.params.release.smoothed.next();
            self.instrument.try_handle_command(&audio_backend::SynthCmd::SetEnvRelease {
            envelope_id: None,
            release,
            });
        }
        
        // Update pan
        if self.params.pan.smoothed.is_smoothing() {
            self.instrument.set_pan(self.params.pan.smoothed.next());
        }
    }
}

impl ClapPlugin for HiHat {
    const CLAP_ID: &'static str = "com.blight-synth.hi-hat";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A hi-hat drum synthesizer");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::Instrument,
        ClapFeature::Drum,
        ClapFeature::Synthesizer,
        ClapFeature::Mono,
    ];
}

impl Vst3Plugin for HiHat {
    const VST3_CLASS_ID: [u8; 16] = *b"BlightHiHat00000";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Drum,
        Vst3SubCategory::Synth,
    ];
}
