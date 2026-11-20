//! Manual on/off switches for machines (`Action::SetMachineEnabled`): a player can shut off any active machine from
//! its panel (`web/src/ui/machine.ts`) to save electricity or fuel.
//!
//! State: a quarry reuses `Quarry::paused`; every other switchable machine stores its anchor in `Factory::manual_off`
//! (sorted by `(x, y, z)` for determinism and O(log n) lookup, saved from version 80 and pruned when a machine is
//! removed). Switched-off machines draw 0 kW, burn no fuel, produce nothing, report `Status::Off` (or `MinerStatus::Off`
//! / `LabStatus::Off` / `QuarryStatus::Paused`), keep their wires attached, and return `None` in efficiency samples so
//! analytics and stall lessons ignore them.
//!
//! To allow a new machine kind to switch: add its `Slot` arm to `Factory::can_switch` and check `sorted_contains` in
//! `power.rs` and `tick.rs`.

use crate::bytes::{ByteReader, ByteWriter};
use crate::math::IVec3;

use super::lab::LabStatus;
use super::links::Slot;
use super::miner::MinerStatus;
use super::process::{Energy, Pick, Status};
use super::Factory;

/// Status line for a machine the player switched off in its panel.
pub const SWITCHED_OFF: &str = "Switched off";

/// Key ordering anchors deterministically in `Factory::manual_off`.
#[inline]
fn key(p: IVec3) -> (i32, i32, i32) {
    (p.x, p.y, p.z)
}

/// Whether `pos` is in a sorted `manual_off` slice.
#[inline]
pub(super) fn is_listed(off: &[IVec3], pos: IVec3) -> bool {
    !off.is_empty() && off.binary_search_by_key(&key(pos), |&p| key(p)).is_ok()
}

impl Factory {
    /// Whether the machine at `pos` has an on/off switch in its panel (active machines, not passive stores or filters).
    pub fn can_switch(&self, pos: IVec3) -> bool {
        match self.at.get(&pos) {
            Some(Slot::Miner(_) | Slot::Generator(_) | Slot::Lab(_) | Slot::Quarry(_)) => true,
            Some(&Slot::Process(i)) => {
                let p = &self.processors[i as usize];
                !matches!(p.spec.pick, Pick::Store | Pick::Load | Pick::Unload | Pick::Land | Pick::Depot)
                    && p.energy() != Energy::Beam
            }
            _ => false,
        }
    }

    /// Whether the machine at `pos` is manually switched off (or paused, for a quarry).
    pub fn is_off(&self, pos: IVec3) -> bool {
        match self.at.get(&pos) {
            Some(&Slot::Quarry(i)) => self.quarries[i as usize].paused,
            Some(_) => self.anchor_of(pos).is_some_and(|a| is_listed(&self.manual_off, a)),
            None => false,
        }
    }

    /// Switches the machine at `pos` on or off; does nothing for machines without a switch.
    pub fn set_enabled(&mut self, pos: IVec3, on: bool) {
        if !self.can_switch(pos) {
            return;
        }
        if let Some(&Slot::Quarry(i)) = self.at.get(&pos) {
            let q = &mut self.quarries[i as usize];
            q.set(q.width, q.depth, !on);
            return;
        }
        let Some(anchor) = self.anchor_of(pos) else { return };
        match (on, self.manual_off.binary_search_by_key(&key(anchor), |&p| key(p))) {
            (false, Err(i)) => self.manual_off.insert(i, anchor),
            (true, Ok(i)) => {
                self.manual_off.remove(i);
                self.clear_off_status(anchor);
            }
            _ => {}
        }
    }

    /// Drops `pos` from `manual_off` when a machine is removed.
    pub(super) fn prune_manual_off(&mut self, pos: IVec3) {
        if let Ok(i) = self.manual_off.binary_search_by_key(&key(pos), |&p| key(p)) {
            self.manual_off.remove(i);
        }
    }

    pub(crate) fn write_manual_off(&self, w: &mut ByteWriter) {
        w.count(self.manual_off.len());
        for &p in &self.manual_off {
            w.ivec3(p);
        }
    }

    pub(crate) fn read_manual_off(&mut self, r: &mut ByteReader) -> Option<()> {
        let n = r.count()?;
        for _ in 0..n {
            let p = r.ivec3()?;
            if self.can_switch(p) && self.anchor_of(p) == Some(p) {
                if let Err(i) = self.manual_off.binary_search_by_key(&key(p), |&x| key(x)) {
                    self.manual_off.insert(i, p);
                }
            }
        }
        Some(())
    }

    /// Resets `Off` status right away when switched back on so readouts update before the next tick finishes.
    fn clear_off_status(&mut self, anchor: IVec3) {
        match self.at.get(&anchor) {
            Some(&Slot::Process(i)) => {
                let p = &mut self.processors[i as usize];
                if p.status == Status::Off {
                    p.status = if p.spec.pick == Pick::Chosen && p.recipe.is_none() {
                        Status::NoRecipe
                    } else {
                        Status::NoInput
                    };
                }
            }
            Some(&Slot::Miner(i)) => {
                let m = &mut self.miners[i as usize];
                if m.status == MinerStatus::Off {
                    m.status = if m.deposit.is_some() { MinerStatus::Running } else { MinerStatus::NoDeposit };
                }
            }
            Some(&Slot::Lab(i)) => {
                let l = &mut self.labs[i as usize];
                if l.status == LabStatus::Off {
                    l.status = LabStatus::NoResearch;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
