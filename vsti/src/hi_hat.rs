use std::sync::Arc;

use audio_backend::InstrumentTrait;
use nih_plug::prelude::*;

pub struct HiHat {
    params: Arc<HiHatParams>,
    sample_rate: f32,
    instrument: audio_backend::HiHat,
}

#[derive(Params)]
struct HiHatParams {}

impl Default for HiHatParams {
    fn default() -> Self {
        Self {}
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
    const NAME: &'static str = "Hi-Hat";
    const VENDOR: &'static str = "PigsR";
    const URL: &'static str = "https://yourwebsite.com";

    const EMAIL: &'static str = "jp.alvarezl@gmail.com";

    const VERSION: &'static str = "0.1.0";

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    type SysExMessage = ();

    type BackgroundTask = ();

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Initialize all voices with correct sample rate
        self.sample_rate = buffer_config.sample_rate as f32;
        self.instrument = audio_backend::HiHat::new(0, 0.0, self.sample_rate);

        true
    }

    fn params(&self) -> std::sync::Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut next_event = context.next_event();

        // Process MIDI events for this sample
        while let Some(event) = next_event {
            match event {
                NoteEvent::NoteOn { note, velocity, .. } => {
                    // Velocity appears to be normalized already
                    self.instrument.note_on(note, (velocity * 127.0) as u8);
                }
                NoteEvent::NoteOff { .. } => {
                    self.instrument.note_off();
                }
                _ => {}
            }

            next_event = context.next_event();
        }

        // Get mutable slices for both channels
        let channels = buffer.as_slice();
        let (left_buf, right_buf) = channels.split_at_mut(1);
        let left_buf = &mut left_buf[0];
        let right_buf = &mut right_buf[0];

        self.instrument
            .process(left_buf, right_buf, self.sample_rate);

        ProcessStatus::Normal
    }
}

impl Vst3Plugin for HiHat {
    const VST3_CLASS_ID: [u8; 16] = *b"HiHat00000000000";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Drum, Vst3SubCategory::Synth];
}
