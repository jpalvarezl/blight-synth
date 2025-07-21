use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::models::{Event, Instrument, SampleData};

pub const DEFAULT_PHRASE_LENGTH: usize = 16;
pub const DEFAULT_CHAIN_LENGTH: usize = 16;
pub const MAX_TRACKS: usize = 8;

pub const EMPTY_PHRASE_SLOT: usize = usize::MAX;
pub const EMPTY_CHAIN_SLOT: usize = usize::MAX;

/// A Phrase is a fixed-size musical block for a single track.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Encode, Decode)]
pub struct Phrase {
    pub events: [Event; DEFAULT_PHRASE_LENGTH],
}

impl Default for Phrase {
    fn default() -> Self {
        Phrase {
            events: [Event::default(); DEFAULT_PHRASE_LENGTH],
        }
    }
}

impl Phrase {
    pub fn from_events<I>(events: I) -> Self
    where
        I: IntoIterator<Item = Event>,
    {
        let mut events_iter = events.into_iter();
        Phrase {
            events: std::array::from_fn(|_| events_iter.next().unwrap_or_default()),
        }
    }
}

/// A Chain is a fixed-size "playlist" of phrases.
/// We use a sentinel value (usize::MAX) to represent an empty slot for size efficiency.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Chain {
    pub phrase_indices: [usize; DEFAULT_CHAIN_LENGTH],
}

impl Default for Chain {
    fn default() -> Self {
        Chain {
            phrase_indices: [EMPTY_PHRASE_SLOT; DEFAULT_CHAIN_LENGTH],
        }
    }
}

/// The top-level Song struct aggregates all components.
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Song {
    pub name: String,
    pub initial_bpm: u16,
    pub initial_speed: u16, // Also known as Ticks Per Line (TPL)

    /// The master arrangement grid. Each row is a step in the song,
    /// and each column is a track. The value is an index into the `chain_bank`.
    /// We use a sentinel value (usize::MAX) for empty slots.
    pub arrangement: Vec<[usize; MAX_TRACKS]>,

    /// A bank containing all unique phrases used in the song.
    pub phrase_bank: Vec<Phrase>,

    /// A bank containing all unique chains used in the song.
    pub chain_bank: Vec<Chain>,

    /// A bank containing all instruments.
    pub instrument_bank: Vec<Instrument>,

    /// A bank containing all raw sample data for sample-based instruments.
    pub sample_bank: Vec<SampleData>,
}

impl Song {
    pub fn new(name: impl Into<String>) -> Self {
        Song {
            name: name.into(),
            initial_bpm: 120,
            initial_speed: 6,
            arrangement: vec![[EMPTY_CHAIN_SLOT; MAX_TRACKS]],
            phrase_bank: vec![Phrase::default()],
            chain_bank: vec![Chain::default()],
            instrument_bank: vec![],
            sample_bank: vec![],
        }
    }
}
