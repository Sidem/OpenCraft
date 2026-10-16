//! Which items the players' machines have ever made (core, saved since 53, kept in `Settlements::made`): the harbour
//! offers only those. Noted from the tick's `Produced` events (`Sim::step`), so whatever reports what it makes counts
//! (processors, miners, quarries, perks); hand crafting does not.

use crate::bytes::{ByteReader, ByteWriter};
use crate::item::{ItemId, ITEM_COUNT};
use crate::sim::SimEvent;

const WORDS: usize = ITEM_COUNT.div_ceil(64);

/// A bit per item.
#[derive(Clone, PartialEq, Debug)]
pub struct Made {
    bits: [u64; WORDS],
}

impl Default for Made {
    fn default() -> Self {
        Made { bits: [0; WORDS] }
    }
}

impl Made {
    /// Notes what `events` say was made.
    pub fn note(&mut self, events: &[SimEvent]) {
        for e in events {
            if let SimEvent::Produced { item, .. } = e {
                self.insert(*item);
            }
        }
    }

    pub fn insert(&mut self, item: ItemId) {
        if let Some(w) = self.bits.get_mut(usize::from(item.0) / 64) {
            *w |= 1 << (item.0 % 64);
        }
    }

    pub fn has(&self, item: ItemId) -> bool {
        self.bits.get(usize::from(item.0) / 64).is_some_and(|w| w >> (item.0 % 64) & 1 == 1)
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(WORDS);
        self.bits.iter().for_each(|&b| w.u64(b));
    }

    /// Reads what `write_state` wrote; words past this build's items are dropped, missing ones are empty.
    pub fn read_state(r: &mut ByteReader) -> Option<Made> {
        let mut made = Made::default();
        for k in 0..r.count()? {
            let word = r.u64()?;
            if let Some(b) = made.bits.get_mut(k) {
                *b = word;
            }
        }
        Some(made)
    }
}
