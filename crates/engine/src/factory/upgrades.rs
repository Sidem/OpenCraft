//! Upgrading tiered machines in place with kits (docs/TECH_TREE.md section 6): a machine's stripe colour
//! is its Mk, and the kit of the next colour raises it one step, keeping direction, contents, links and
//! power. Kits per step come from the family (`tiers.rs`); research gates each step
//! (`Unlock::Upgrade`, checked by `Action::Upgrade`). Breaking returns the upgraded machine, never kits.
//!
//! Invariants: an upgrade only changes the machine's `tier`, by exactly one, to a tier its family has.
//!
//! To add a tier's kit: its item (`item.rs`, wearing `tex::stripe(tier)`), a `KITS` entry and its hand
//! or assembler recipe.

use crate::block::{BlockId, BELT, MINER};
use crate::item::{ItemId, GREEN_KIT};
use crate::math::IVec3;

use super::links::Slot;
use super::tiers;
use super::Factory;

/// Tier colours as 0xRRGGBB, Mk1 first: red, green, blue, violet, gold (the science packs' colours).
pub const TIER_COLOURS: [u32; 5] = [0xd94a3d, 0x4caf50, 0x3f7fd9, 0x9a5bd6, 0xe0b02f];

/// The kit that raises a machine to each tier (Mk1 needs none). Blue, violet and gold come with their eras.
pub const KITS: [ItemId; 2] = [ItemId::NONE, GREEN_KIT];

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
    /// The family block and tier of the tiered machine at `pos` (a belt of any shape, a miner or a
    /// processor).
    pub fn tiered_at(&self, pos: IVec3) -> Option<(BlockId, u8)> {
        match *self.at.get(&pos)? {
            Slot::Belt(i) => Some((BELT, self.belts[i as usize].tier)),
            Slot::Miner(i) => Some((MINER, self.miners[i as usize].tier)),
            Slot::Process(i) => Some((self.processors[i as usize].spec.block, self.processors[i as usize].tier)),
            _ => None,
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
        if self.next_upgrade(pos).is_none() {
            return false;
        }
        match self.at[&pos] {
            Slot::Belt(i) => self.belts[i as usize].tier += 1,
            Slot::Miner(i) => self.miners[i as usize].tier += 1,
            Slot::Process(i) => self.processors[i as usize].tier += 1,
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
mod tests;
