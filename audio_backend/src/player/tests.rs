use std::sync::{Arc, Mutex};

use dsp::{
    id::{EffectId, InstrumentId, NoteEvent, NoteId},
    EffectInstallError, EffectInstallErrorKind, InstrumentTrait, MonoEffect, SynthCmd,
    VoiceEffects,
};
use engine::{EventAdmissionErrorKind, InstrumentCmd};
use sequencer::models::{
    Chain, EffectType, Event, Phrase, SongRow, EMPTY_CHAIN_SLOT, EMPTY_PHRASE_SLOT,
};

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraceEvent {
    On { frame: usize, pitch: u8 },
    Release { frame: usize },
}

struct TraceInstrument {
    id: InstrumentId,
    frame: usize,
    pitch: Option<u8>,
    tail_frames: usize,
    trace: Arc<Mutex<Vec<TraceEvent>>>,
}

impl InstrumentTrait for TraceInstrument {
    fn id(&self) -> InstrumentId {
        self.id
    }

    fn note_on(&mut self, event: NoteEvent) {
        self.pitch = Some(event.pitch);
        self.tail_frames = 0;
        self.trace.lock().unwrap().push(TraceEvent::On {
            frame: self.frame,
            pitch: event.pitch,
        });
    }

    fn note_off(&mut self, _note_id: NoteId) {
        self.release();
    }

    fn all_notes_off(&mut self) {
        self.release();
    }

    fn process(&mut self, left: &mut [f32], right: &mut [f32], _sample_rate: f32) {
        for (left, right) in left.iter_mut().zip(right) {
            let value = if let Some(pitch) = self.pitch {
                f32::from(pitch) / 127.0
            } else if self.tail_frames != 0 {
                self.tail_frames -= 1;
                0.125
            } else {
                0.0
            };
            *left += value;
            *right += value;
            self.frame += 1;
        }
    }

    fn set_pan(&mut self, _pan: f32) {}

    fn add_effect(&mut self, effect: Box<dyn MonoEffect>) -> Result<(), EffectInstallError> {
        Err(EffectInstallError::new(
            EffectInstallErrorKind::UnsupportedForPolyphonicInstrument,
            effect,
        ))
    }

    fn add_voice_effects(&mut self, effects: VoiceEffects) -> VoiceEffects {
        effects
    }

    fn set_effect_parameter(&mut self, _effect_id: EffectId, _param_index: u32, _value: f32) {}

    fn try_handle_command(&mut self, _command: &SynthCmd) -> bool {
        false
    }
}

impl TraceInstrument {
    fn release(&mut self) {
        self.pitch = None;
        self.tail_frames = 8;
        self.trace
            .lock()
            .unwrap()
            .push(TraceEvent::Release { frame: self.frame });
    }
}

fn cell(note: u8, instrument_id: u8) -> Event {
    Event {
        note,
        volume: 100,
        instrument_id,
        effect: EffectType::Arpeggio,
        effect_param: 0,
    }
}

fn song_with_track_phrases(sample_bpm: u16, speed: u16, track_phrases: Vec<Vec<Event>>) -> Song {
    let mut song = Song::new("event integration");
    song.initial_bpm = sample_bpm;
    song.initial_speed = speed;
    song.phrase_bank = track_phrases
        .iter()
        .map(|events| Phrase::from_events(events.iter().copied()))
        .collect();
    song.chain_bank = (0..track_phrases.len())
        .map(|index| {
            let mut phrases = [EMPTY_PHRASE_SLOT; DEFAULT_CHAIN_LENGTH];
            phrases.fill(index);
            Chain::new(phrases)
        })
        .collect();
    let mut chains = [EMPTY_CHAIN_SLOT; MAX_TRACKS];
    for (index, chain) in chains.iter_mut().take(track_phrases.len()).enumerate() {
        *chain = index;
    }
    song.arrangement = vec![SongRow::new(chains)];
    song
}

fn player_with_trace(song: Song, sample_rate: f64) -> (Player, Arc<Mutex<Vec<TraceEvent>>>) {
    let trace = Arc::new(Mutex::new(Vec::new()));
    let mut player = Player::new(Arc::new(song), sample_rate);
    player.handle_command(
        InstrumentCmd::AddInstrument {
            instrument: Box::new(TraceInstrument {
                id: 1,
                frame: 0,
                pitch: None,
                tail_frames: 0,
                trace: trace.clone(),
            }),
        }
        .into(),
        &mut engine::DropRetireSink,
    );
    (player, trace)
}

fn render_partitions(partitions: &[usize]) -> (Vec<f32>, Vec<TraceEvent>) {
    let song = song_with_track_phrases(25, 1, vec![vec![cell(60, 1), cell(64, 0)]]);
    let (mut player, trace) = player_with_trace(song, 100.0); // ten frames per tick
    player.loop_enabled = true;
    assert_eq!(player.play(), TimingAdvanceStatus::Complete);
    let mut rendered = Vec::new();
    for &frames in partitions {
        let mut left = vec![0.0; frames];
        let mut right = vec![0.0; frames];
        assert!(player.process(&mut left, &mut right, 100.0).is_complete());
        rendered.extend(left);
    }
    let events = trace.lock().unwrap().clone();
    (rendered, events)
}

fn patterned(total: usize, pattern: &[usize]) -> Vec<usize> {
    let mut result = Vec::new();
    let mut remaining = total;
    let mut index = 0;
    while remaining != 0 {
        let frames = pattern[index % pattern.len()].min(remaining);
        result.push(frames);
        remaining -= frames;
        index += 1;
    }
    result
}

#[test]
fn exact_and_block_boundary_offsets_are_applied_at_the_boundary() {
    let song = song_with_track_phrases(25, 1, vec![vec![cell(60, 1), cell(64, 0)]]);
    let (mut player, trace) = player_with_trace(song, 10.0); // one frame per tick
    player.play();

    let mut left = [0.0; 1];
    let mut right = [0.0; 1];
    player.process(&mut left, &mut right, 10.0);
    assert!(trace.lock().unwrap().is_empty());

    player.process(&mut left, &mut right, 10.0);
    player.process(&mut left, &mut right, 10.0);
    assert_eq!(
        *trace.lock().unwrap(),
        [
            TraceEvent::On {
                frame: 1,
                pitch: 60
            },
            TraceEvent::On {
                frame: 2,
                pitch: 64
            }
        ]
    );
}

#[test]
fn event_positions_and_output_are_partition_invariant_including_host_chunks() {
    let total = 5_000;
    let expected = render_partitions(&[total]);
    for partitions in [
        patterned(total, &[256]),
        patterned(total, &[1, 511, 17, 2_047]),
        vec![1; total],
        vec![MAX_RENDER_SLICE_FRAMES, total - MAX_RENDER_SLICE_FRAMES],
    ] {
        assert_eq!(render_partitions(&partitions), expected);
    }
}

#[test]
fn same_offset_tracks_follow_stable_track_and_canonical_order() {
    let song = song_with_track_phrases(25, 1, vec![vec![cell(60, 1)], vec![cell(67, 1)]]);
    let (mut player, trace) = player_with_trace(song, 10.0);
    player.play();
    let mut left = [0.0; 2];
    let mut right = [0.0; 2];

    player.process(&mut left, &mut right, 10.0);

    assert_eq!(
        *trace.lock().unwrap(),
        [
            TraceEvent::On {
                frame: 1,
                pitch: 60
            },
            TraceEvent::On {
                frame: 1,
                pitch: 67
            }
        ]
    );
}

#[test]
fn tracker_note_off_releases_at_its_tick_and_tail_keeps_rendering() {
    let song = song_with_track_phrases(
        25,
        1,
        vec![vec![
            cell(60, 1),
            cell(NoteSentinelValues::NoteOff as u8, 0),
        ]],
    );
    let (mut player, trace) = player_with_trace(song, 10.0);
    player.play();
    let mut left = [0.0; 4];
    let mut right = [0.0; 4];

    player.process(&mut left, &mut right, 10.0);

    assert_eq!(
        *trace.lock().unwrap(),
        [
            TraceEvent::On {
                frame: 1,
                pitch: 60
            },
            TraceEvent::Release { frame: 2 }
        ]
    );
    assert_eq!(left[0], 0.0);
    assert!(left[1] > 0.4);
    assert_eq!(&left[2..], &[0.125, 0.125]);
}

#[test]
fn stopped_transport_admits_live_attack_release_and_renders_tail() {
    let song = song_with_track_phrases(25, 1, vec![vec![]]);
    let (mut player, trace) = player_with_trace(song, 10.0);
    assert!(!player.is_playing());

    player.handle_command(
        InstrumentCmd::NoteOn {
            instrument_id: 1,
            note: 72,
            velocity: 100,
        }
        .into(),
        &mut engine::DropRetireSink,
    );
    let mut attack_left = [0.0; 4];
    let mut right = [0.0; 4];
    assert!(player
        .process(&mut attack_left, &mut right, 10.0)
        .is_complete());
    assert!(attack_left.iter().all(|sample| *sample > 0.0));

    player.handle_command(
        InstrumentCmd::NoteOff { instrument_id: 1 }.into(),
        &mut engine::DropRetireSink,
    );
    let mut release_left = [0.0; 4];
    assert!(player
        .process(&mut release_left, &mut right, 10.0)
        .is_complete());
    let mut tail_left = [0.0; 4];
    assert!(player
        .process(&mut tail_left, &mut right, 10.0)
        .is_complete());

    assert_eq!(
        *trace.lock().unwrap(),
        [
            TraceEvent::On {
                frame: 0,
                pitch: 72
            },
            TraceEvent::Release { frame: 4 }
        ]
    );
    assert_eq!(release_left, [0.125; 4]);
    assert_eq!(tail_left, [0.125; 4]);
}

#[test]
fn same_block_live_attack_then_release_coalesces_without_a_stuck_attack() {
    let song = song_with_track_phrases(25, 1, vec![vec![]]);
    let (mut player, trace) = player_with_trace(song, 10.0);
    player.handle_command(
        InstrumentCmd::NoteOn {
            instrument_id: 1,
            note: 72,
            velocity: 100,
        }
        .into(),
        &mut engine::DropRetireSink,
    );
    player.handle_command(
        InstrumentCmd::NoteOff { instrument_id: 1 }.into(),
        &mut engine::DropRetireSink,
    );
    let mut left = [0.0; 4];
    let mut right = [0.0; 4];

    assert!(player.process(&mut left, &mut right, 10.0).is_complete());

    assert_eq!(*trace.lock().unwrap(), [TraceEvent::Release { frame: 0 }]);
    assert_eq!(left, [0.125; 4]);
}

#[test]
fn end_stops_with_recovery_while_loop_keeps_clock_continuous() {
    let song = song_with_track_phrases(25, 1, vec![vec![cell(60, 1)]]);
    let (mut ending, ending_trace) = player_with_trace(song.clone(), 10.0);
    ending.play();
    let mut left = vec![0.0; 257];
    let mut right = vec![0.0; 257];
    ending.process(&mut left, &mut right, 10.0);
    assert!(!ending.is_playing());
    assert_eq!(ending.position, PlayerPosition::default());
    assert!(ending_trace
        .lock()
        .unwrap()
        .contains(&TraceEvent::Release { frame: 256 }));

    let (mut looping, loop_trace) = player_with_trace(song, 10.0);
    looping.loop_enabled = true;
    looping.play();
    looping.process(&mut left, &mut right, 10.0);
    assert!(looping.is_playing());
    let mut boundary_left = [0.0; 1];
    let mut boundary_right = [0.0; 1];
    looping.process(&mut boundary_left, &mut boundary_right, 10.0);
    assert!(loop_trace.lock().unwrap().contains(&TraceEvent::On {
        frame: 257,
        pitch: 60
    }));
}

#[test]
fn tpl_applies_to_the_row_being_entered_and_bpm_to_the_next_interval() {
    let mut speed = cell(60, 1);
    speed.effect = EffectType::SetSpeedOrBPM;
    speed.effect_param = 2;
    let song = song_with_track_phrases(10, 6, vec![vec![speed, cell(61, 0)]]);
    let (mut player, trace) = player_with_trace(song, 10.0); // phases 2.5, 5, 7.5
    player.play();
    let mut left = [0.0; 9];
    let mut right = [0.0; 9];
    player.process(&mut left, &mut right, 10.0);
    assert!(trace.lock().unwrap().contains(&TraceEvent::On {
        frame: 8,
        pitch: 61
    }));

    let mut tempo = cell(62, 1);
    tempo.effect = EffectType::SetSpeedOrBPM;
    tempo.effect_param = 40;
    let song = song_with_track_phrases(10, 1, vec![vec![tempo, cell(63, 0)]]);
    let (mut player, trace) = player_with_trace(song, 20.0); // first=5, then +1.25 => 7
    player.play();
    let mut left = [0.0; 8];
    let mut right = [0.0; 8];
    player.process(&mut left, &mut right, 20.0);
    assert!(trace.lock().unwrap().contains(&TraceEvent::On {
        frame: 7,
        pitch: 63
    }));
}

#[test]
fn admission_overflow_is_fail_closed_observable_and_recovery_is_reserved() {
    let song = song_with_track_phrases(25, 1, vec![vec![cell(60, 1)], vec![cell(67, 1)]]);
    let trace = Arc::new(Mutex::new(Vec::new()));
    let mut player = Player::with_event_capacity(Arc::new(song), 10.0, 1);
    player.handle_command(
        InstrumentCmd::AddInstrument {
            instrument: Box::new(TraceInstrument {
                id: 1,
                frame: 0,
                pitch: None,
                tail_frames: 0,
                trace: trace.clone(),
            }),
        }
        .into(),
        &mut engine::DropRetireSink,
    );
    player.play();
    let mut left = [0.0; 2];
    let mut right = [0.0; 2];

    let status = player.process(&mut left, &mut right, 10.0);

    assert!(matches!(
        status.events,
        EventLaneStatus::AdmissionRejected(error)
            if error.kind() == EventAdmissionErrorKind::OrdinaryCapacityExceeded
    ));
    assert_eq!(*trace.lock().unwrap(), [TraceEvent::Release { frame: 0 }]);
    assert!(left.iter().all(|sample| *sample == 0.125));
    assert!(!player.is_playing());
    assert_eq!(player.position, PlayerPosition::default());
}

struct CollectRetired(Vec<RetiredState>);

impl RetireSink for CollectRetired {
    fn retire(&mut self, state: RetiredState) {
        self.0.push(state);
    }
}

#[test]
fn load_and_play_song_retire_displaced_snapshots_for_nrt_drop() {
    for play_immediately in [false, true] {
        let previous = Arc::new(Song::new("previous"));
        let weak_previous = Arc::downgrade(&previous);
        let mut player = Player::new(previous, 48_000.0);
        let mut retired = CollectRetired(Vec::new());
        let song = Arc::new(Song::new("next"));
        let command = if play_immediately {
            SequencerCmd::PlaySong { song }.into()
        } else {
            SequencerCmd::LoadSong { song }.into()
        };

        player.handle_command(command, &mut retired);

        assert_eq!(player.is_playing(), play_immediately);
        assert_eq!(retired.0.len(), 1);
        assert!(weak_previous.upgrade().is_some());
        drop(retired);
        assert!(weak_previous.upgrade().is_none());
    }
}

#[test]
fn invalid_replacement_retires_rejection_and_keeps_old_song_recoverable() {
    let old_song = Arc::new(Song::new("valid old song"));
    let mut player = Player::new(old_song, 48_000.0);
    assert_eq!(player.play(), TimingAdvanceStatus::Complete);
    let mut retired = CollectRetired(Vec::new());
    let mut invalid = Song::new("invalid replacement");
    invalid.initial_bpm = 0;

    player.handle_command(
        SequencerCmd::PlaySong {
            song: Arc::new(invalid),
        }
        .into(),
        &mut retired,
    );

    assert_eq!(player.song.name, "valid old song");
    assert_eq!(
        player.timing_status,
        TimingAdvanceStatus::InvalidConfiguration
    );
    assert!(!player.is_playing());
    assert_eq!(retired.0.len(), 1);

    player.handle_command(TransportCmd::PlayLastSong.into(), &mut retired);
    assert!(player.is_playing());
    assert_eq!(player.timing_status, TimingAdvanceStatus::Complete);
}

#[test]
fn tracker_instrument_cache_is_exactly_max_tracks_and_directly_indexed() {
    let adapter = tracker_engine_adapter::TrackerEngineAdapter::new();
    assert_eq!(adapter.track_instruments().len(), MAX_TRACKS);
    assert_eq!(
        adapter.track_instruments(),
        [NO_INSTRUMENT as InstrumentId; MAX_TRACKS]
    );
}
