//! World-wide item discovery codex (deterministic core state, saved from `SAVE_VERSION` 83):
//! one fixed bitset of items ever obtained in this world (`obtained`), folded from `SimEvent`s at
//! the end of `Sim::step`. Periodic table elements (`data.rs`) and achievements (`achievements.rs`)
//! are pure read-only views over this bitset — no per-feature state, no second source of truth.
//!
//! Invariants: bits are monotonic (once obtained, never cleared); `write_state` is compact (only
//! up to the highest non-zero word) and canonical so co-op peers share the exact same hash.
//! To extend: append rows to `data::FORMS` / `data::ELEMENTS` or `achievements::ACHIEVEMENTS`.

use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::Factory;
use crate::item::ItemId;
use crate::math::IVec3;
use crate::sim::{Away, PlayerCore, SimEvent};

pub mod achievements;
pub mod data;

pub use achievements::ACHIEVEMENTS;
pub use data::{element_info, ELEMENTS, FORMS};

/// 16 × 64 = 1,024 item ids fit in the fixed bitset (item ids are currently < 450).
const WORDS: usize = 16;

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Codex {
    obtained: [u64; WORDS],
}

impl Codex {
    /// Marks `item` as obtained in this world. Returns `true` if this was the first time.
    pub fn record(&mut self, item: ItemId) -> bool {
        if item == ItemId::NONE {
            return false;
        }
        let idx = item.0 as usize;
        let word = idx / 64;
        if word >= WORDS {
            return false;
        }
        let mask = 1u64 << (idx % 64);
        let fresh = (self.obtained[word] & mask) == 0;
        self.obtained[word] |= mask;
        fresh
    }

    /// Whether `item` has ever been obtained in this world.
    #[inline]
    pub fn has(&self, item: ItemId) -> bool {
        let idx = item.0 as usize;
        let word = idx / 64;
        word < WORDS && (self.obtained[word] & (1u64 << (idx % 64))) != 0
    }

    /// Folds this tick's core events (`Produced`, `Gained`, `Crafted`, `Launched`) into the bitset.
    pub fn note(&mut self, events: &[SimEvent]) {
        for &e in events {
            match e {
                SimEvent::Produced { item, .. }
                | SimEvent::Gained { item, .. }
                | SimEvent::Crafted { item, .. }
                | SimEvent::Launched { item, .. } => {
                    self.record(item);
                }
                _ => {}
            }
        }
    }

    /// Seeds a codex for worlds loaded from saves older than version 83, using items already in
    /// player inventories and factory storage boxes.
    pub fn seed_from(players: &[Option<PlayerCore>], away: &[Away], factory: &Factory) -> Codex {
        let mut c = Codex::default();
        for p in players.iter().flatten() {
            for s in p.inventory.open().iter().filter(|s| !s.is_empty()) {
                c.record(s.item);
            }
            if !p.inventory.offhand.is_empty() {
                c.record(p.inventory.offhand.item);
            }
        }
        for a in away {
            for s in a.inventory.open().iter().filter(|s| !s.is_empty()) {
                c.record(s.item);
            }
            if !a.inventory.offhand.is_empty() {
                c.record(a.inventory.offhand.item);
            }
        }
        for s in factory.stock_totals(IVec3::ZERO, i32::MAX / 4) {
            c.record(s.item);
        }
        c
    }

    /// Number of distinct items ever obtained (cheap monotonic version for UI redraw checks).
    pub fn obtained_count(&self) -> u32 {
        self.obtained.iter().map(|w| w.count_ones()).sum()
    }

    /// `(obtained_forms, total_forms)` for periodic table element `z`.
    pub fn element_forms_done(&self, z: u8) -> (usize, usize) {
        let (mut done, mut total) = (0, 0);
        for f in FORMS.iter().filter(|f| f.z == z) {
            total += 1;
            if self.has(f.item) {
                done += 1;
            }
        }
        (done, total)
    }

    /// Whether at least one form of element `z` has been obtained.
    pub fn element_discovered(&self, z: u8) -> bool {
        FORMS.iter().any(|f| f.z == z && self.has(f.item))
    }

    /// Number of distinct reachable elements discovered so far.
    pub fn discovered_elements(&self) -> usize {
        ELEMENTS.iter().filter(|e| self.element_discovered(e.z)).count()
    }

    /// Canonical compact encoding: count of non-zero trailing words (`u8`), then those `u64` words.
    pub fn write_state(&self, w: &mut ByteWriter) {
        let len = self.obtained.iter().rposition(|&x| x != 0).map_or(0, |i| i + 1);
        w.u8(len as u8);
        for &word in &self.obtained[..len] {
            w.u64(word);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Codex> {
        let len = r.u8()? as usize;
        if len > WORDS {
            return None;
        }
        let mut c = Codex::default();
        for word in &mut c.obtained[..len] {
            *word = r.u64()?;
        }
        Some(c)
    }
}

#[cfg(test)]
mod tests;
