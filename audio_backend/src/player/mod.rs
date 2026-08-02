mod tracker_engine_adapter;

use std::sync::Arc;

use engine::{
    BoundedEventAdmission, EngineEvent, EventAdmissionError, EventProcessError, EventProducerId,
    InstrumentCmd, OrdinaryEventBlockStatus, ProducerAdmissionStatus, RecoveryAdmissionStatus,
    RetireSink, RetiredState, TimestampedEvent,
};
use sequencer::{
    models::{
        EffectType, Event, NoteSentinelValues, Song, DEFAULT_CHAIN_LENGTH, DEFAULT_PHRASE_LENGTH,
        EMPTY_CHAIN_SLOT, EMPTY_PHRASE_SLOT, MAX_TRACKS, NO_INSTRUMENT,
    },
    timing::{TickBoundary, TickTempo, TimingAdvanceStatus, TimingError, TimingState},
};

use crate::{id::InstrumentId, Command, SequencerCmd, TransportCmd, MAX_RENDER_SLICE_FRAMES};

const TRACKER_PRODUCER: EventProducerId = EventProducerId::new(1);
const LIVE_PRODUCER: EventProducerId = EventProducerId::new(2);
const RECOVERY_PRODUCER: EventProducerId = EventProducerId::new(3);
const MAX_TICKS_PER_SLICE: usize = MAX_RENDER_SLICE_FRAMES;
// One explicit instrument cell can release the prior instrument and then emit
// its note/release operation. This is the structural per-tick/track maximum.
const MAX_TRACKER_EVENTS_PER_SLICE: usize = MAX_TICKS_PER_SLICE * MAX_TRACKS * 2;
// The device host drains at most this many commands before one callback. Keep
// this local constant in sync with that host budget; direct callers get the
// same explicit bound.
const MAX_LIVE_EVENTS_PER_BLOCK: usize = 64;
const DEFAULT_EVENT_CAPACITY: usize = MAX_TRACKER_EVENTS_PER_SLICE + MAX_LIVE_EVENTS_PER_BLOCK;

/// Compact callback-visible event-lane outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventLaneStatus {
    Complete,
    LiveInputCapacityExceeded,
    PreparedEventCapacityExceeded,
    SourceSequenceExhausted,
    AdmissionRejected(EventAdmissionError),
    ProcessRejected(EventProcessError),
}

/// Combined status for one exact render slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerProcessStatus {
    pub timing: TimingAdvanceStatus,
    pub events: EventLaneStatus,
}

impl PlayerProcessStatus {
    const fn new(timing: TimingAdvanceStatus, events: EventLaneStatus) -> Self {
        Self { timing, events }
    }

    pub const fn complete() -> Self {
        Self::new(TimingAdvanceStatus::Complete, EventLaneStatus::Complete)
    }

    pub const fn is_complete(self) -> bool {
        matches!(self.timing, TimingAdvanceStatus::Complete)
            && matches!(self.events, EventLaneStatus::Complete)
    }

    /// Preserve the first compact failure while folding internally chunked
    /// slices into one host-callback result.
    #[cfg(feature = "device-host")]
    pub(crate) const fn combine(self, next: Self) -> Self {
        let timing = if matches!(self.timing, TimingAdvanceStatus::Complete) {
            next.timing
        } else {
            self.timing
        };
        let events = if matches!(self.events, EventLaneStatus::Complete) {
            next.events
        } else {
            self.events
        };
        Self { timing, events }
    }
}

/// Holds the playback position for a single track.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TrackPosition {
    pub chain_step: u8,
    pub phrase_step: u8,
}

/// Holds the complete playback position state for the entire song.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerPosition {
    pub song_step: usize,
    pub tick_counter: u32,
    pub track_positions: [TrackPosition; MAX_TRACKS],
}

impl PlayerPosition {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Default for PlayerPosition {
    fn default() -> Self {
        Self {
            song_step: 0,
            tick_counter: 0,
            track_positions: [TrackPosition::default(); MAX_TRACKS],
        }
    }
}

/// Fixed-memory tracker/live producer and host-owned composite scheduler.
///
/// Worst-case callback work is 4096 tick boundaries, eight tracks per row, at
/// most two tracker events per track/tick, 64 queued live events, one canonical
/// bounded sort, and one segmented engine render. All vectors and admission
/// storage are prepared in `new` and never grow on the callback.
pub struct Player {
    song: Arc<Song>,
    timing: TimingState,
    timing_status: TimingAdvanceStatus,
    // Ticks per line advances tracker rows; it never changes tick spacing.
    tpl: u32,
    position: PlayerPosition,
    is_playing: bool,
    loop_enabled: bool,
    engine_adapter: tracker_engine_adapter::TrackerEngineAdapter,
    tick_boundaries: Vec<TickBoundary>,
    tracker_events: Vec<TimestampedEvent>,
    queued_live_events: Vec<EngineEvent>,
    live_events: Vec<TimestampedEvent>,
    admission: BoundedEventAdmission,
    tracker_sequence: u64,
    live_sequence: u64,
    recovery_sequence: u64,
    pending_recovery: bool,
    live_overflow: bool,
    process_status: PlayerProcessStatus,
}

impl Player {
    pub fn new(song: Arc<Song>, sample_rate: f64) -> Self {
        Self::with_event_capacity(song, sample_rate, DEFAULT_EVENT_CAPACITY)
    }

    fn with_event_capacity(song: Arc<Song>, sample_rate: f64, event_capacity: usize) -> Self {
        let max_ticks =
            u32::try_from(MAX_TICKS_PER_SLICE).expect("the prepared render-slice bound fits u32");
        let timing = TimingState::prepare(sample_rate, song.initial_bpm as f64, max_ticks)
            .unwrap_or_else(|_| TimingState::new_with_bpm(sample_rate, song.initial_bpm as f64));
        let timing_status = timing.status();
        let tpl = normalized_tpl(song.initial_speed as u32);
        let admission = BoundedEventAdmission::prepare(
            event_capacity,
            &[TRACKER_PRODUCER, LIVE_PRODUCER],
            RECOVERY_PRODUCER,
        )
        .expect("static event producers and bounded storage are valid");

        Self {
            song,
            timing,
            timing_status,
            tpl,
            position: PlayerPosition::default(),
            is_playing: false,
            loop_enabled: false,
            engine_adapter: tracker_engine_adapter::TrackerEngineAdapter::new(),
            tick_boundaries: vec![TickBoundary::default(); MAX_TICKS_PER_SLICE],
            tracker_events: Vec::with_capacity(MAX_TRACKER_EVENTS_PER_SLICE),
            queued_live_events: Vec::with_capacity(MAX_LIVE_EVENTS_PER_BLOCK),
            live_events: Vec::with_capacity(MAX_LIVE_EVENTS_PER_BLOCK),
            admission,
            tracker_sequence: 0,
            live_sequence: 0,
            recovery_sequence: 0,
            pending_recovery: false,
            live_overflow: false,
            process_status: PlayerProcessStatus::new(timing_status, EventLaneStatus::Complete),
        }
    }

    #[cfg(feature = "device-host")]
    pub(crate) fn instrument_capacity(&self) -> usize {
        self.engine_adapter.instrument_capacity()
    }

    pub fn play(&mut self) -> TimingAdvanceStatus {
        self.timing_status = self.timing.status();
        self.is_playing = self.timing_status == TimingAdvanceStatus::Complete;
        self.process_status.timing = self.timing_status;
        self.timing_status
    }

    pub(crate) fn is_playing(&self) -> bool {
        self.is_playing
    }

    #[cfg(feature = "device-host")]
    pub(crate) const fn process_status(&self) -> PlayerProcessStatus {
        self.process_status
    }

    fn stop_transport(&mut self, discard_earlier_live_events: bool) {
        self.is_playing = false;
        self.position.reset();
        self.timing_status = match self.timing.reset() {
            Ok(()) => TimingAdvanceStatus::Complete,
            Err(error) => timing_error_status(error),
        };
        self.process_status.timing = self.timing_status;
        self.pending_recovery = true;
        if discard_earlier_live_events {
            // Preserve command FIFO semantics at offset zero: live events before
            // stop are cancelled, while events queued after stop still audition.
            self.queued_live_events.clear();
            self.live_overflow = false;
        }
    }

    pub fn stop(&mut self) {
        self.stop_transport(true);
    }

    fn set_song(&mut self, song: Arc<Song>, retired: &mut impl RetireSink) -> bool {
        if self.timing.set_bpm(song.initial_bpm as f64).is_err() || self.timing.reset().is_err() {
            self.stop_transport(true);
            self.timing_status = TimingAdvanceStatus::InvalidConfiguration;
            self.process_status.timing = self.timing_status;
            retired.retire(RetiredState::Prepared(song));
            return false;
        }

        self.timing_status = TimingAdvanceStatus::Complete;
        self.tpl = normalized_tpl(song.initial_speed as u32);
        self.position.reset();
        self.admission.reset();
        self.tracker_sequence = 0;
        self.live_sequence = 0;
        self.recovery_sequence = 0;
        let displaced = std::mem::replace(&mut self.song, song);
        retired.retire(RetiredState::Prepared(displaced));
        true
    }

    fn load_song(&mut self, song: Arc<Song>, retired: &mut impl RetireSink) {
        dsp::rt_debug_log!("Loading song: {}", song.name);
        self.stop_transport(true);
        if self.set_song(song, retired) {
            self.engine_adapter.clear_instruments(retired);
        }
    }

    pub fn handle_command(&mut self, command: Command, retired: &mut impl RetireSink) {
        match command {
            Command::Sequencer(SequencerCmd::LoadSong { song }) => self.load_song(song, retired),
            Command::Sequencer(SequencerCmd::PlaySong { song }) => {
                dsp::rt_debug_log!("Playing song: {}", song.name);
                self.stop_transport(true);
                if self.set_song(song, retired) {
                    self.play();
                }
            }
            Command::Transport(TransportCmd::StopSong) => self.stop(),
            Command::Transport(TransportCmd::SetLooping { enabled }) => {
                self.loop_enabled = enabled;
            }
            Command::Transport(TransportCmd::PlayLastSong) => {
                self.play();
            }
            Command::Instrument(InstrumentCmd::NoteOn {
                instrument_id,
                note,
                velocity,
            }) => self.queue_live_event(EngineEvent::NoteOn {
                instrument_id,
                note: dsp::NoteEvent::from_pitch(note, velocity),
            }),
            Command::Instrument(InstrumentCmd::NoteOff { instrument_id }) => {
                self.queue_live_event(EngineEvent::InstrumentAllNotesOff { instrument_id });
            }
            Command::Instrument(command) => self
                .engine_adapter
                .handle_engine_command(command.into(), retired),
            Command::Mixer(command) => self
                .engine_adapter
                .handle_engine_command(command.into(), retired),
        }
    }

    fn queue_live_event(&mut self, event: EngineEvent) {
        if let EngineEvent::InstrumentAllNotesOff { instrument_id } = event {
            // Every queued live event is intentionally at offset zero. Canonical
            // release precedence would otherwise turn a FIFO NoteOn→release
            // pair into release→attack and leave a stuck note. Coalesce attacks
            // and duplicate releases for this instrument that have zero-frame
            // lifetime; a later attack remains after this release and therefore
            // still follows canonical ordering.
            self.queued_live_events.retain(|queued| {
                !matches!(
                    queued,
                    EngineEvent::NoteOn {
                        instrument_id: queued_instrument,
                        ..
                    } | EngineEvent::InstrumentAllNotesOff {
                        instrument_id: queued_instrument,
                    } if *queued_instrument == instrument_id
                )
            });
        }
        if self.queued_live_events.len() == MAX_LIVE_EVENTS_PER_BLOCK {
            self.live_overflow = true;
        } else {
            self.queued_live_events.push(event);
        }
    }

    /// Process one exact common planar-buffer prefix. Tracker and live events
    /// are admitted together and applied at their offsets; voices/effect tails
    /// render even when tracker transport is stopped.
    pub fn process(
        &mut self,
        left: &mut [f32],
        right: &mut [f32],
        sample_rate: f32,
    ) -> PlayerProcessStatus {
        let frame_count = left.len().min(right.len());
        if frame_count == 0 {
            self.engine_adapter.process(left, right, sample_rate);
            self.process_status =
                PlayerProcessStatus::new(self.timing_status, EventLaneStatus::Complete);
            return self.process_status;
        }

        self.tracker_events.clear();
        self.live_events.clear();
        let mut staged_position = self.position;
        let mut staged_tpl = self.tpl;
        let mut staged_playing = self.is_playing;
        let mut staged_track_instruments = self.engine_adapter.track_instruments();
        let mut staged_tracker_sequence = self.tracker_sequence;
        let mut staged_live_sequence = self.live_sequence;
        let mut tracker_end_offset = None;
        let mut timing_failed = false;
        let mut local_event_failure = None;

        if self.is_playing {
            let song = &self.song;
            let loop_enabled = self.loop_enabled;
            let mut planned_position = self.position;
            let mut planned_tpl = self.tpl;
            let mut planned_playing = true;
            let timing = &mut self.timing;
            let boundaries = &mut self.tick_boundaries;
            let timing_advance = timing.advance_ticks(frame_count, boundaries, |_| {
                if !planned_playing {
                    return TickTempo::Unchanged;
                }
                let tempo = apply_row_timing(song, &planned_position, &mut planned_tpl);
                if advance_position(
                    &mut planned_position,
                    planned_tpl,
                    song.arrangement.len(),
                    loop_enabled,
                ) {
                    planned_playing = false;
                }
                tempo
            });
            self.timing_status = timing_advance.status;

            if timing_advance.status == TimingAdvanceStatus::Complete {
                for boundary in &self.tick_boundaries[..timing_advance.ticks_emitted as usize] {
                    if !staged_playing {
                        break;
                    }
                    let _ = apply_row_timing(&self.song, &staged_position, &mut staged_tpl);
                    if staged_position.tick_counter == 0 {
                        if let Err(error) = emit_current_row(
                            &self.song,
                            &staged_position,
                            boundary.sample_offset,
                            &mut staged_track_instruments,
                            &mut staged_tracker_sequence,
                            &mut self.tracker_events,
                        ) {
                            local_event_failure = Some(error);
                            break;
                        }
                    }
                    if advance_position(
                        &mut staged_position,
                        staged_tpl,
                        self.song.arrangement.len(),
                        self.loop_enabled,
                    ) {
                        staged_playing = false;
                        tracker_end_offset = Some(boundary.sample_offset);
                    }
                }
            } else {
                staged_playing = false;
                staged_position.reset();
                timing_failed = true;
            }
        }

        if self.live_overflow {
            local_event_failure = Some(EventLaneStatus::LiveInputCapacityExceeded);
        } else if !timing_failed && local_event_failure.is_none() {
            for event in &self.queued_live_events {
                let Some(next_sequence) = staged_live_sequence.checked_add(1) else {
                    local_event_failure = Some(EventLaneStatus::SourceSequenceExhausted);
                    break;
                };
                self.live_events.push(TimestampedEvent::new(
                    0,
                    LIVE_PRODUCER,
                    staged_live_sequence,
                    *event,
                ));
                staged_live_sequence = next_sequence;
            }
        }

        self.admission.begin_block(frame_count);
        let mut admission_rejection = None;
        if !timing_failed && local_event_failure.is_none() {
            for status in [
                self.admission
                    .submit_producer(TRACKER_PRODUCER, &self.tracker_events),
                self.admission
                    .submit_producer(LIVE_PRODUCER, &self.live_events),
            ] {
                if let ProducerAdmissionStatus::Rejected(error) = status {
                    admission_rejection = Some(error);
                }
            }
        }

        let needs_recovery = self.pending_recovery
            || tracker_end_offset.is_some()
            || timing_failed
            || local_event_failure.is_some()
            || admission_rejection.is_some();
        let recovery_offset = tracker_end_offset.unwrap_or(0);
        let mut recovery_staged = false;
        if needs_recovery {
            if let Some(next_sequence) = self.recovery_sequence.checked_add(1) {
                recovery_staged = matches!(
                    self.admission
                        .request_all_notes_off(recovery_offset, self.recovery_sequence),
                    RecoveryAdmissionStatus::Staged
                );
                if recovery_staged {
                    self.recovery_sequence = next_sequence;
                }
            } else {
                local_event_failure = Some(EventLaneStatus::SourceSequenceExhausted);
            }
        }

        let finalized = self.admission.finish_block();
        let ordinary_status = finalized.ordinary_status();
        let events = finalized.events();
        let mut event_status = local_event_failure.unwrap_or(EventLaneStatus::Complete);
        if let OrdinaryEventBlockStatus::Rejected(error) = ordinary_status {
            event_status = EventLaneStatus::AdmissionRejected(error);
        }

        let process_result =
            self.engine_adapter
                .process_with_events(left, right, sample_rate, events);
        if let Err(error) = process_result {
            // Validation is transactional: render the untouched voices/tails
            // without events rather than exposing stale output.
            self.engine_adapter.process(left, right, sample_rate);
            event_status = EventLaneStatus::ProcessRejected(error);
            self.pending_recovery = true;
        } else {
            if matches!(ordinary_status, OrdinaryEventBlockStatus::Accepted { .. })
                && !timing_failed
                && local_event_failure.is_none()
            {
                self.position = staged_position;
                self.tpl = staged_tpl;
                self.is_playing = staged_playing;
                self.engine_adapter
                    .set_track_instruments(staged_track_instruments);
                self.tracker_sequence = staged_tracker_sequence;
                self.live_sequence = staged_live_sequence;
                if tracker_end_offset.is_some() && !staged_playing {
                    self.timing_status = match self.timing.reset() {
                        Ok(()) => TimingAdvanceStatus::Complete,
                        Err(error) => timing_error_status(error),
                    };
                }
            } else {
                self.is_playing = false;
                self.position.reset();
            }
            if recovery_staged {
                self.pending_recovery = false;
            }
        }

        self.queued_live_events.clear();
        self.live_overflow = false;
        self.process_status = PlayerProcessStatus::new(self.timing_status, event_status);
        self.process_status
    }
}

fn timing_error_status(error: TimingError) -> TimingAdvanceStatus {
    match error {
        TimingError::PositionOverflow => TimingAdvanceStatus::PositionOverflow,
        TimingError::InvalidSampleRate
        | TimingError::InvalidBpm
        | TimingError::TickIntervalOutOfRange
        | TimingError::ZeroTickCapacity => TimingAdvanceStatus::InvalidConfiguration,
    }
}

fn normalized_tpl(tpl: u32) -> u32 {
    tpl.max(1)
}

/// Apply Fxx at a row's first tick. F01..F1F changes TPL for the row that is
/// beginning now; F20..FF changes BPM for the interval after this exact tick.
/// F00 is ignored. Tracks are scanned in ascending stable index, so the last
/// applicable command wins deterministically.
fn apply_row_timing(song: &Song, position: &PlayerPosition, tpl: &mut u32) -> TickTempo {
    if position.tick_counter != 0 {
        return TickTempo::Unchanged;
    }
    let mut tempo = TickTempo::Unchanged;
    for track_index in 0..MAX_TRACKS {
        let Some(event) = event_for_track(song, position, track_index) else {
            continue;
        };
        if !matches!(event.effect, EffectType::SetSpeedOrBPM) || event.effect_param == 0 {
            continue;
        }
        if event.effect_param < 0x20 {
            *tpl = u32::from(event.effect_param);
        } else {
            tempo = TickTempo::SetBpm(f64::from(event.effect_param));
        }
    }
    tempo
}

fn emit_current_row(
    song: &Song,
    position: &PlayerPosition,
    sample_offset: usize,
    track_instruments: &mut [InstrumentId; MAX_TRACKS],
    sequence: &mut u64,
    output: &mut Vec<TimestampedEvent>,
) -> Result<(), EventLaneStatus> {
    for (track_index, cached_instrument) in track_instruments.iter_mut().enumerate() {
        let Some(event) = event_for_track(song, position, track_index) else {
            continue;
        };
        let explicit_instrument = event.instrument_id as InstrumentId;
        let instrument_id = if explicit_instrument == NO_INSTRUMENT as InstrumentId {
            *cached_instrument
        } else {
            let previous = *cached_instrument;
            *cached_instrument = explicit_instrument;
            if previous != NO_INSTRUMENT as InstrumentId {
                push_tracker_event(
                    sample_offset,
                    sequence,
                    EngineEvent::InstrumentAllNotesOff {
                        instrument_id: previous,
                    },
                    output,
                )?;
            }
            explicit_instrument
        };

        if event.note == NoteSentinelValues::NoteOff as u8 {
            push_tracker_event(
                sample_offset,
                sequence,
                EngineEvent::InstrumentAllNotesOff { instrument_id },
                output,
            )?;
        } else if event.note != NoteSentinelValues::NoNote as u8 {
            let velocity = if event.volume == 0 { 255 } else { event.volume };
            push_tracker_event(
                sample_offset,
                sequence,
                EngineEvent::NoteOn {
                    instrument_id,
                    note: dsp::NoteEvent::from_pitch(event.note, velocity),
                },
                output,
            )?;
        }
    }
    Ok(())
}

fn push_tracker_event(
    sample_offset: usize,
    sequence: &mut u64,
    event: EngineEvent,
    output: &mut Vec<TimestampedEvent>,
) -> Result<(), EventLaneStatus> {
    let next_sequence = sequence
        .checked_add(1)
        .ok_or(EventLaneStatus::SourceSequenceExhausted)?;
    if output.len() == output.capacity() {
        return Err(EventLaneStatus::PreparedEventCapacityExceeded);
    }
    output.push(TimestampedEvent::new(
        sample_offset,
        TRACKER_PRODUCER,
        *sequence,
        event,
    ));
    *sequence = next_sequence;
    Ok(())
}

fn event_for_track<'a>(
    song: &'a Song,
    position: &PlayerPosition,
    track_index: usize,
) -> Option<&'a Event> {
    let row = song.arrangement.get(position.song_step)?;
    let chain_index = row.chain_indices[track_index];
    if chain_index == EMPTY_CHAIN_SLOT {
        return None;
    }
    let chain = song.chain_bank.get(chain_index)?;
    let track_position = position.track_positions[track_index];
    let phrase_index = chain.phrase_indices[track_position.chain_step as usize];
    if phrase_index == EMPTY_PHRASE_SLOT {
        return None;
    }
    song.phrase_bank
        .get(phrase_index)?
        .events
        .get(track_position.phrase_step as usize)
}

/// Advance after one tick. Returns true only when non-looping playback reached
/// song end. TPL changes are applied before this comparison, at the row's first
/// tick; BPM never participates in row progression.
fn advance_position(
    position: &mut PlayerPosition,
    tpl: u32,
    arrangement_len: usize,
    loop_enabled: bool,
) -> bool {
    position.tick_counter += 1;
    if position.tick_counter < normalized_tpl(tpl) {
        return false;
    }
    position.tick_counter = 0;

    let mut song_step_needs_advancing = false;
    for track_position in &mut position.track_positions {
        track_position.phrase_step += 1;
        if track_position.phrase_step >= DEFAULT_PHRASE_LENGTH as u8 {
            track_position.phrase_step = 0;
            track_position.chain_step += 1;
            if track_position.chain_step >= DEFAULT_CHAIN_LENGTH as u8 {
                track_position.chain_step = 0;
                song_step_needs_advancing = true;
            }
        }
    }

    if song_step_needs_advancing {
        position.song_step += 1;
        if position.song_step >= arrangement_len {
            position.reset();
            return !loop_enabled;
        }
    }
    false
}

#[cfg(test)]
mod tests;
