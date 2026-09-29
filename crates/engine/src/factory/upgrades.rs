//! Upgrading tiered machines in place with kits (docs/TECH_TREE.md section 6): a machine's stripe colour
//! is its Mk, and the kit of the next colour raises it one step, keeping direction, contents, links and
//! power. Kits per step come from the family (`tiers.rs`); research gates each step
//! (`Unlock::Upgrade`, checked by `Action::Upgrade`). Breaking returns the upgraded machine, never kits.
//!
//! Invariants: an upgrade only changes the machine's `tier`, by exactly one, to a tier its family has.
//!
//! To add a tier's kit: its item (`item.rs`, wearing `tex::stripe(tier)`), a `KITS` entry and its hand
//! or assembler recipe.

use crate::block::{BlockId, BELT, GENERATOR, LAB, MINER, POLE, PUMP, QUARRY, STORAGE};
use crate::item::{ItemId, BLUE_KIT, GREEN_KIT};
use crate::math::IVec3;

use super::links::Slot;
use super::pipes::Part;
use super::tiers;
use super::Factory;

/// Tier colours as 0xRRGGBB, Mk1 first: red, green, blue, violet, gold (the science packs' colours).
pub const TIER_COLOURS: [u32; 5] = [0xd94a3d, 0x4caf50, 0x3f7fd9, 0x9a5bd6, 0xe0b02f];

/// The kit that raises a machine to each tier (Mk1 needs none). Violet and gold come with their eras.
pub const KITS: [ItemId; 3] = [ItemId::NONE, GREEN_KIT, BLUE_KIT];

/// One upgrade step: the family's block, the tier it reaches, and the kits it takes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Step {
    pub block: BlockId,
    pub tier: u8,
    pub kit: ItemId,
    pub kits: u32,
}

/// The kit that raises machines to `tier`.
pub fn kit(tier: u8) -> Option<ItemId> {
    KITS.get(tier as usize).copied().filter(|&k| k != ItemId::NONE)
}

/// The tier a kit item raises machines to, if it is a kit.
pub fn kit_tier(item: ItemId) -> Option<u8> {
    (1..KITS.len()).find(|&t| KITS[t] == item).map(|t| t as u8)
}

impl Factory {
    /// The family block and tier of the tiered machine at `pos` (a belt of any shape, a miner, a
    /// processor, a box, a generator, a pole, a lab, a pump or a quarry).
    pub fn tiered_at(&self, pos: IVec3) -> Option<(BlockId, u8)> {
        match *self.at.get(&pos)? {
            Slot::Belt(i) => Some((BELT, self.belts[i as usize].tier)),
            Slot::Miner(i) => Some((MINER, self.miners[i as usize].tier)),
            Slot::Process(i) => {
                let p = &self.processors[i as usize];
                tiers::family(p.spec.block).map(|_| (p.spec.block, p.tier))
            }
            Slot::Storage(i) => Some((STORAGE, self.storages[i as usize].tier)),
            Slot::Generator(i) => Some((GENERATOR, self.generators[i as usize].tier)),
            Slot::Pole(i) => Some((POLE, self.poles[i as usize].tier)).filter(|t| t.1 != super::pole::CABLE_TIER),
            Slot::Lab(i) => Some((LAB, self.labs[i as usize].tier)),
            Slot::Pipe(i) => {
                (self.pipework[i as usize].part == Part::Pump).then_some((PUMP, self.pipework[i as usize].tier))
            }
            Slot::Quarry(i) => Some((QUARRY, self.quarries[i as usize].tier)),
            Slot::Router(_) => None,
        }
    }

    /// The next upgrade step of the machine at `pos`; `None` when it isn't tiered or is at its top tier.
    pub fn next_upgrade(&self, pos: IVec3) -> Option<Step> {
        let (block, tier) = self.tiered_at(pos)?;
        let family = tiers::family(block)?;
        let next = tier + 1;
        if usize::from(next) >= family.items.len() {
            return None;
        }
        Some(Step { block, tier: next, kit: kit(next)?, kits: family.kits })
    }

    /// Raises the machine at `pos` one tier (the caller has taken the kits). False if it can't go higher.
    pub fn upgrade(&mut self, pos: IVec3) -> bool {
        let Some((_, tier)) = self.tiered_at(pos).filter(|_| self.next_upgrade(pos).is_some()) else { return false };
        self.set_tier(pos, tier + 1);
        true
    }

    /// Makes the machine at `pos` tier `tier` (an upgrade, or a placed tier item), resizing what depends on it.
    pub(super) fn set_tier(&mut self, pos: IVec3, tier: u8) {
        match self.at[&pos] {
            Slot::Belt(i) => self.belts[i as usize].tier = tier,
            Slot::Miner(i) => self.miners[i as usize].tier = tier,
            Slot::Process(i) => self.processors[i as usize].tier = tier,
            Slot::Storage(i) => self.storages[i as usize].set_tier(tier),
            Slot::Generator(i) => self.generators[i as usize].tier = tier,
            Slot::Pole(i) => self.poles[i as usize].tier = tier,
            Slot::Lab(i) => self.labs[i as usize].set_tier(tier),
            Slot::Pipe(i) => self.pipework[i as usize].tier = tier,
            Slot::Quarry(i) => self.quarries[i as usize].tier = tier,
            Slot::Router(_) => {}
        }
        // A tier can change how a machine is powered or wired (the Mk3 smelter is electric, a pylon
        // reaches further): link it again.
        self.dirty = true;
    }
}
#[cfg(test)]
mod tests;
