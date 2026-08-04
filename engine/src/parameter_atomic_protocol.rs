//! Load-bearing ADR 0005 memory orders shared by the implementation and model.
//!
//! Keeping these constants in one source file means the deterministic Loom model
//! and the shipped store cannot silently drift to different orderings.

use std::sync::atomic::Ordering;

pub(crate) const GENERATION_CLOSE: Ordering = Ordering::Release;
pub(crate) const GENERATION_OBSERVE: Ordering = Ordering::Acquire;
pub(crate) const SLOT_CAS_SUCCESS: Ordering = Ordering::Relaxed;
pub(crate) const SLOT_CAS_FAILURE: Ordering = Ordering::Relaxed;
pub(crate) const DIRTY_PUBLISH: Ordering = Ordering::Release;
pub(crate) const DIRTY_CONSUME: Ordering = Ordering::Acquire;
pub(crate) const SLOT_CONSUME: Ordering = Ordering::Relaxed;
pub(crate) const APPLIED_PUBLISH: Ordering = Ordering::Release;
pub(crate) const APPLIED_OBSERVE: Ordering = Ordering::Acquire;
