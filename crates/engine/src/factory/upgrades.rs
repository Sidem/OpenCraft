//! Upgrading tiered machines in place with kits (docs/TECH_TREE.md section 6): a machine's stripe colour
//! is its Mk, and the kit of the next colour raises it one step, keeping direction, contents, links and
//! power. Kits per step come from the family (`tiers.rs`); research gates each step
//! (`Unlock::Upgrade`, checked by `Action::Upgrade`). Breaking returns the upgraded machine, never kits.
//!
//! Invariants: an upgrade only changes the machine's `tier`, by exactly one, to a tier its family has.
//!
//! To add a tier's kit: its item (`item.rs`, wearing `tex::stripe(tier)`), a `KITS` entry and its hand
//! or assembler recipe.

use crate::block::{BlockId, BELT, GENERATOR, LAB, LIFT, MINER, POLE, PUMP, QUARRY, STORAGE, UNDERPASS_IN};
use crate::item::{ItemId, BLUE_KIT, GOLD_KIT, GREEN_KIT, VIOLET_KIT};
use crate::math::IVec3;

use super::belt_shape::Shape;
use super::links::Slot;
use super::pipes::Part;
use super::tiers;
use super::Factory;

/// Tier colours as 0xRRGGBB, Mk1 first: red, green, blue, violet, gold (the science packs' colours).
pub const TIER_COLOURS: [u32; 5] = [0xd94a3d, 0x4caf50, 0x3f7fd9, 0x9a5bd6, 0xe0b02f];

/// The kit that raises a machine to each tier (Mk1 needs none).
pub const KITS: [ItemId; 5] = [ItemId::NONE, GREEN_KIT, BLUE_KIT, VIOLET_KIT, GOLD_KIT];

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
        self.tiered_slot(*self.at.get(&pos)?)
    }

    /// What the machine at `cell` is called, with its Mk if it has tiers ("Miner Mk3", not the block's
    /// "Miner Mk1"): the one name every readout, panel and map uses for a placed machine.
    pub fn name_at(&self, cell: IVec3) -> Option<String> {
        self.at.get(&cell).map(|&s| self.name_of(s))
    }

    /// [`Self::name_at`] for a machine named by its slot.
    pub(crate) fn name_of(&self, slot: Slot) -> String {
        match self.tiered_slot(slot) {
            // A lift is a belt of the belt's tiers that is not called one.
            Some((_, tier)) if matches!(slot, Slot::Belt(i) if self.belts[i as usize].shape == Shape::Lift) => {
                format!("{} Mk{}", crate::block::def(LIFT).name, u32::from(tier) + 1)
            }
            Some((block, tier)) => tiers::name(block, tier),
            None => crate::block::def(self.block_in(slot)).name.to_string(),
        }
    }

    fn tiered_slot(&self, slot: Slot) -> Option<(BlockId, u8)> {
        match slot {
            Slot::Belt(i) => {
                let b = &self.belts[i as usize];
                Some((if b.shape.is_pass() { UNDERPASS_IN } else { BELT }, b.tier))
            }
            // A coal miner has no family: no kit raises it.
            Slot::Miner(i) => {
                let m = &self.miners[i as usize];
                (!m.is_coal()).then_some((MINER, m.tier))
            }
            Slot::Process(i) => {
                let p = &self.processors[i as usize];
                tiers::family(p.spec.block).map(|_| (p.spec.block, p.tier))
            }
            Slot::Storage(i) => Some((STORAGE, self.storages[i as usize].tier)),
            Slot::Generator(i) => Some((GENERATOR, self.generators[i as usize].tier)),
            Slot::Pole(i) => Some((POLE, self.poles[i as usize].tier))
                .filter(|t| t.1 != super::pole::CABLE_TIER && t.1 != super::pole::LONG_TIER),
            Slot::Lab(i) => Some((LAB, self.labs[i as usize].tier)),
            Slot::Pipe(i) => {
                (self.pipework[i as usize].part == Part::Pump).then_some((PUMP, self.pipework[i as usize].tier))
            }
            Slot::Quarry(i) => Some((QUARRY, self.quarries[i as usize].tier)),
            Slot::Router(_) | Slot::Sensor(_) | Slot::Rail(_) | Slot::Node(_) => None,
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

    /// The kits the next upgrade of every machine at `cells` takes together (an underpass takes more than a belt).
    pub fn upgrade_kits(&self, cells: impl IntoIterator<Item = IVec3>) -> u32 {
        cells.into_iter().filter_map(|p| self.next_upgrade(p)).map(|s| s.kits).sum()
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
            Slot::Router(_) | Slot::Sensor(_) | Slot::Rail(_) | Slot::Node(_) => {}
        }
        // A tier can change how a machine is powered or wired (the Mk3 smelter is electric, a pylon
        // reaches further): link it again.
        self.dirty = true;
    }
}
#[cfg(test)]
mod tests;
