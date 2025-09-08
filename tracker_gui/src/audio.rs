use crate::audio_utils::map_waveform_to_backend;
use audio_backend::{BlightAudio, SequencerCmd, TransportCmd};
use sequencer::models::{AudioEffect, InstrumentData, Song};
use std::sync::Arc;

pub struct AudioManager {
    pub audio: Option<BlightAudio>,
    pub is_playing: bool,
}

impl Default for AudioManager {
    fn default() -> Self {
        Self {
            audio: None,
            is_playing: false,
        }
    }
}

impl AudioManager {
    pub fn init_audio(&mut self, song: &Song) {
        if self.audio.is_none() {
            match BlightAudio::with_song(Arc::new(song.clone())) {
                Ok(mut audio) => {
                    self.hydrate_from_song(&mut audio, song);
                    self.audio = Some(audio);
                    log::info!("Audio system initialized successfully");
                }
                Err(e) => {
                    log::error!("Failed to initialize audio system: {}", e);
                }
            }
        }
    }

    pub fn reset_with_song(&mut self, song: &Song) {
        match BlightAudio::with_song(Arc::new(song.clone())) {
            Ok(mut audio) => {
                self.hydrate_from_song(&mut audio, song);
                self.audio = Some(audio);
                self.is_playing = false;
                log::info!("Audio system reset for loaded song");
            }
            Err(e) => {
                log::error!("Failed to reset audio system: {}", e);
            }
        }
    }

    pub fn play_song(&mut self, song: &Song) {
        self.init_audio(song);

        if let Some(audio) = &mut self.audio {
            audio.send_command(
                SequencerCmd::PlaySong {
                    song: Arc::new(song.clone()),
                }
                .into(),
            );
            self.is_playing = true;
            log::info!("Playing song: {}", song.name);
        }
    }

    pub fn stop_song(&mut self) {
        if let Some(audio) = &mut self.audio {
            audio.send_command(TransportCmd::StopSong.into());
            self.is_playing = false;
            log::info!("Stopped song");
        }
    }

    pub fn toggle_playback(&mut self, song: &Song) {
        if self.is_playing {
            self.stop_song();
        } else {
            self.play_song(song);
        }
    }

    pub fn hydrate_from_song(&self, audio: &mut BlightAudio, song: &Song) {
        for inst in &song.instrument_bank {
            let id = audio_backend::id::InstrumentId::from(inst.id as u32);
            match &inst.data {
                InstrumentData::SimpleOscillator(p) => {
                    let wf = map_waveform_to_backend(p.waveform);
                    let instrument = audio
                        .get_instrument_factory()
                        .create_oscillator_with_waveform(id, 0.0, wf);
                    audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());

                    // Apply mono insert effects
                    for eff in p.audio_effects.iter() {
                        match eff {
                            AudioEffect::Reverb {
                                mix,
                                decay_time,
                                room_size,
                                diffusion,
                                damping,
                            } => {
                                let mut r = audio.get_effect_factory().create_mono_reverb();
                                // Clamp to safe ranges consistent with UI/backend
                                let mx = (*mix).clamp(0.0, 1.0);
                                let dec = (*decay_time).clamp(0.0, 1.0);
                                let rs = (*room_size).clamp(0.5, 2.0);
                                let damp = (*damping).clamp(0.0, 1.0);
                                let diff = (*diffusion).clamp(0.0, 1.0);
                                use audio_backend::effects::ReverbParameter as RP;
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *r,
                                    RP::Mix.as_index(),
                                    mx,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *r,
                                    RP::Decay.as_index(),
                                    dec,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *r,
                                    RP::RoomSize.as_index(),
                                    rs,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *r,
                                    RP::Damping.as_index(),
                                    damp,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *r,
                                    RP::Diffusion.as_index(),
                                    diff,
                                );
                                audio.send_command(
                                    SequencerCmd::AddEffectToInstrument {
                                        instrument_id: id,
                                        effect: r,
                                    }
                                    .into(),
                                );
                            }
                            AudioEffect::Delay {
                                time,
                                num_taps,
                                feedback,
                                mix,
                            } => {
                                let mut d = audio.get_effect_factory().create_mono_delay(
                                    *time,
                                    *num_taps as usize,
                                    *feedback,
                                    *mix,
                                );
                                use audio_backend::effects::DelayParameter as DP;
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *d,
                                    DP::Time.as_index(),
                                    *time,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *d,
                                    DP::NumTaps.as_index(),
                                    *num_taps as f32,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *d,
                                    DP::Feedback.as_index(),
                                    *feedback,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *d,
                                    DP::Mix.as_index(),
                                    *mix,
                                );
                                audio.send_command(
                                    SequencerCmd::AddEffectToInstrument {
                                        instrument_id: id,
                                        effect: d,
                                    }
                                    .into(),
                                );
                            }
                        }
                    }
                }
                InstrumentData::HiHat(p) => {
                    let instrument = audio.get_instrument_factory().create_hihat(id, 0.0);
                    audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());

                    for eff in p.audio_effects.iter() {
                        match eff {
                            AudioEffect::Reverb {
                                mix,
                                decay_time,
                                room_size,
                                diffusion,
                                damping,
                            } => {
                                let mut r = audio.get_effect_factory().create_mono_reverb();
                                let mx = (*mix).clamp(0.0, 1.0);
                                let dec = (*decay_time).clamp(0.0, 1.0);
                                let rs = (*room_size).clamp(0.5, 2.0);
                                let damp = (*damping).clamp(0.0, 1.0);
                                let diff = (*diffusion).clamp(0.0, 1.0);
                                use audio_backend::effects::ReverbParameter as RP;
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *r,
                                    RP::Mix.as_index(),
                                    mx,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *r,
                                    RP::Decay.as_index(),
                                    dec,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *r,
                                    RP::RoomSize.as_index(),
                                    rs,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *r,
                                    RP::Damping.as_index(),
                                    damp,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *r,
                                    RP::Diffusion.as_index(),
                                    diff,
                                );
                                audio.send_command(
                                    SequencerCmd::AddEffectToInstrument {
                                        instrument_id: id,
                                        effect: r,
                                    }
                                    .into(),
                                );
                            }
                            AudioEffect::Delay {
                                time,
                                num_taps,
                                feedback,
                                mix,
                            } => {
                                let mut d = audio.get_effect_factory().create_mono_delay(
                                    *time,
                                    *num_taps as usize,
                                    *feedback,
                                    *mix,
                                );
                                use audio_backend::effects::DelayParameter as DP;
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *d,
                                    DP::Time.as_index(),
                                    *time,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *d,
                                    DP::NumTaps.as_index(),
                                    *num_taps as f32,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *d,
                                    DP::Feedback.as_index(),
                                    *feedback,
                                );
                                audio_backend::MonoEffect::set_parameter(
                                    &mut *d,
                                    DP::Mix.as_index(),
                                    *mix,
                                );
                                audio.send_command(
                                    SequencerCmd::AddEffectToInstrument {
                                        instrument_id: id,
                                        effect: d,
                                    }
                                    .into(),
                                );
                            }
                        }
                    }
                }
                InstrumentData::KickDrum(p) => {
                    let instrument = audio.get_instrument_factory().create_kick_drum(id, 0.0);
                    audio.send_command(SequencerCmd::AddTrackInstrument { instrument }.into());

                    for eff in p.audio_effects.iter() {
                        match eff {
                            AudioEffect::Reverb { mix, decay_time, room_size, diffusion, damping } => {
                                let mut r = audio.get_effect_factory().create_mono_reverb();
                                let mx = (*mix).clamp(0.0, 1.0);
                                let dec = (*decay_time).clamp(0.0, 1.0);
                                let rs = (*room_size).clamp(0.5, 2.0);
                                let damp = (*damping).clamp(0.0, 1.0);
                                let diff = (*diffusion).clamp(0.0, 1.0);
                                use audio_backend::effects::ReverbParameter as RP;
                                audio_backend::MonoEffect::set_parameter(&mut *r, RP::Mix.as_index(), mx);
                                audio_backend::MonoEffect::set_parameter(&mut *r, RP::Decay.as_index(), dec);
                                audio_backend::MonoEffect::set_parameter(&mut *r, RP::RoomSize.as_index(), rs);
                                audio_backend::MonoEffect::set_parameter(&mut *r, RP::Damping.as_index(), damp);
                                audio_backend::MonoEffect::set_parameter(&mut *r, RP::Diffusion.as_index(), diff);
                                audio.send_command(SequencerCmd::AddEffectToInstrument { instrument_id: id, effect: r }.into());
                            }
                            AudioEffect::Delay { time, num_taps, feedback, mix } => {
                                let mut d = audio.get_effect_factory().create_mono_delay(*time, *num_taps as usize, *feedback, *mix);
                                use audio_backend::effects::DelayParameter as DP;
                                audio_backend::MonoEffect::set_parameter(&mut *d, DP::Time.as_index(), *time);
                                audio_backend::MonoEffect::set_parameter(&mut *d, DP::NumTaps.as_index(), *num_taps as f32);
                                audio_backend::MonoEffect::set_parameter(&mut *d, DP::Feedback.as_index(), *feedback);
                                audio_backend::MonoEffect::set_parameter(&mut *d, DP::Mix.as_index(), *mix);
                                audio.send_command(SequencerCmd::AddEffectToInstrument { instrument_id: id, effect: d }.into());
                            }
                        }
                    }
                }
                _ => {
                    log::warn!(
                        "Skipping unsupported instrument id={} when hydrating audio",
                        inst.id
                    );
                }
            }
        }
    }
}
