use crate::audio::AudioManager;
use crate::ui_components::effect_controls::EffectSync;
use sequencer::models::Song;

use super::backend::ensure_backend_instrument;

#[derive(Default)]
pub struct InstrumentSync {
    pending: Vec<u8>,
}

impl InstrumentSync {
    pub fn queue_rehydrate(&mut self, instrument_id: u8) {
        self.pending.push(instrument_id);
    }

    pub fn ensure_now(&mut self, song: &Song, audio_mgr: &mut AudioManager, instrument_id: u8) {
        if let Some(inst) = song
            .instrument_bank
            .iter()
            .find(|i| i.id as u8 == instrument_id)
        {
            ensure_backend_instrument(audio_mgr, instrument_id, &inst.data);
        }
    }

    pub fn apply_pending(&mut self, song: &Song, audio_mgr: &mut AudioManager) {
        if self.pending.is_empty() {
            return;
        }
        self.pending.sort_unstable();
        self.pending.dedup();
        let ids = self.pending.drain(..).collect::<Vec<_>>();
        for instrument_id in ids {
            self.ensure_now(song, audio_mgr, instrument_id);
        }
    }
}

impl EffectSync for InstrumentSync {
    fn queue_rehydrate(&mut self, instrument_id: u8) {
        self.pending.push(instrument_id);
    }
}
