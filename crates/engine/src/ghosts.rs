//! Ghosts: blocks and machines the player has planned but not built (core state, saved since version 26,
//! so co-op peers share them and construction drones can build them: Milestone 7).
//!
//! A ghost names an anchor cell, a block, a facing and a tier. Placing a ghost (`Action::PlaceGhost`) costs
//! nothing: it takes the block from the item in a slot and needs every cell it would occupy free. Building
//! the block for real on a ghost's anchor (`Sim::place_block`) uses the ghost's facing and clears the
//! ghost; `Action::RemoveGhost` clears one by hand. Ghosts never block anything: the world is untouched
//! until something is built.
//!
//! Invariants: kept sorted by anchor (so the state bytes are canonical), at most [`MAX_GHOSTS`], at most
//! one per anchor. A multi-block machine's ghost covers its footprint (`covers`).
//!
//! The hands that plant ghosts and the outlines that draw them are presentation (`ghost_mode.rs`).

use crate::block::{self, BlockId};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::{footprint, tiers};
use crate::item::ItemId;
use crate::math::IVec3;
use crate::sim::{PlayerId, Sim};

/// Ghosts one world can hold.
pub const MAX_GHOSTS: usize = 4096;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ghost {
    pub pos: IVec3,
    pub block: BlockId,
    pub facing: u8,
    pub tier: u8,
}

impl Ghost {
    /// Every cell the ghost covers, the anchor first.
    pub fn cells(&self) -> Vec<IVec3> {
        footprint::of(self.block).map_or_else(|| vec![self.pos], |f| f.cells(self.pos, self.facing))
    }

    /// The item that builds it (a tiered machine's tier item).
    pub fn item(&self) -> ItemId {
        tiers::item_of(self.block, self.tier).unwrap_or(ItemId::block(self.block))
    }

    fn key(&self) -> (i32, i32, i32) {
        (self.pos.x, self.pos.y, self.pos.z)
    }
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Ghosts {
    list: Vec<Ghost>,
}

impl Ghosts {
    pub fn iter(&self) -> impl Iterator<Item = &Ghost> {
        self.list.iter()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    /// The ghost whose cells include `cell`.
    pub fn covering(&self, cell: IVec3) -> Option<&Ghost> {
        self.list.iter().find(|g| g.cells().contains(&cell))
    }

    /// Adds `g`, replacing any ghost with the same anchor. False when the world is at [`MAX_GHOSTS`].
    pub fn set(&mut self, g: Ghost) -> bool {
        match self.list.binary_search_by_key(&g.key(), Ghost::key) {
            Ok(i) => self.list[i] = g,
            Err(_) if self.list.len() >= MAX_GHOSTS => return false,
            Err(i) => self.list.insert(i, g),
        }
        true
    }

    /// Removes the ghost covering `cell`; false when there is none.
    pub fn remove_at(&mut self, cell: IVec3) -> bool {
        let Some(anchor) = self.covering(cell).map(|g| g.pos) else { return false };
        self.list.retain(|g| g.pos != anchor);
        true
    }

    /// The facing of the ghost of `block` anchored at `pos`, if there is one.
    pub fn facing_for(&self, pos: IVec3, block: BlockId) -> Option<u8> {
        self.at(pos, block).map(|g| g.facing)
    }

    /// Clears the ghost of `block` anchored at `pos` (it has been built).
    pub fn built(&mut self, pos: IVec3, block: BlockId) {
        if self.at(pos, block).is_some() {
            self.list.retain(|g| g.pos != pos);
        }
    }

    /// What building every ghost needs: the items and how many of each, most needed first (tear-down marks
    /// need nothing).
    pub fn needs(&self) -> Vec<(ItemId, u32)> {
        let mut out: Vec<(ItemId, u32)> = Vec::new();
        for g in self.list.iter().filter(|g| g.block != block::AIR) {
            match out.iter_mut().find(|(item, _)| *item == g.item()) {
                Some((_, n)) => *n += 1,
                None => out.push((g.item(), 1)),
            }
        }
        crate::math::sort_small_by_key(&mut out, |&(item, n)| (u32::MAX - n, item.0));
        out
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.list.len());
        for g in &self.list {
            w.ivec3(g.pos);
            w.u8(g.block);
            w.u8(g.facing);
            w.u8(g.tier);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Ghosts> {
        let n = r.count()?;
        if n > MAX_GHOSTS {
            return None;
        }
        let mut ghosts = Ghosts::default();
        for _ in 0..n {
            let g = Ghost { pos: r.ivec3()?, block: r.block()?, facing: r.u8()?, tier: r.u8()? };
            // Damage: a ghost out of order or twice, or a facing that is not a quarter turn.
            if g.facing >= 4 || ghosts.list.last().is_some_and(|l| l.key() >= g.key()) {
                return None;
            }
            ghosts.list.push(g);
        }
        Some(ghosts)
    }

    fn at(&self, pos: IVec3, block: BlockId) -> Option<&Ghost> {
        let i = self.list.binary_search_by_key(&(pos.x, pos.y, pos.z), Ghost::key).ok()?;
        Some(&self.list[i]).filter(|g| g.block == block)
    }
}

impl Sim {
    /// Plants a ghost of the block the item in `slot` places, at `pos` facing `facing`, without using
    /// the item. Nothing happens if the slot is empty or holds no block, a cell it would take is solid
    /// or held by a machine, or the world is full of ghosts.
    pub(crate) fn place_ghost(&mut self, player: PlayerId, pos: IVec3, slot: u8, facing: u8) {
        let Some(Some(core)) = self.players.get(player.0 as usize) else { return };
        let Some(stack) = core.inventory.slots.get(slot as usize).copied() else { return };
        let Some(block) = stack.item.places().filter(|_| !stack.is_empty()) else { return };
        let tier = tiers::placed_by(stack.item).map_or(0, |(_, t)| t);
        self.plant_ghost(pos, block, facing, tier);
    }

    /// Marks the block at `pos` (a multi-block machine by its anchor) for tear-down by drones: a ghost of air.
    /// Nothing happens if there is nothing breakable there or the world is full of ghosts.
    pub(crate) fn mark_removal(&mut self, pos: IVec3) {
        let at = self.factory.footprint_at(pos).map_or(pos, |f| f.0);
        let id = self.world.block_anywhere_or_generate(at);
        if id != block::AIR && block::def(id).break_time >= 0.0 {
            self.ghosts.set(Ghost { pos: at, block: block::AIR, facing: 0, tier: 0 });
        }
    }

    /// Plants a ghost of `block` outright (what a stamped blueprint does). Nothing happens for air, a tier
    /// the block has no item for, a cell it would take being solid or held by a machine, or a full world.
    pub(crate) fn plant_ghost(&mut self, pos: IVec3, block: BlockId, facing: u8, tier: u8) {
        let ghost = Ghost { pos, block, facing: facing % 4, tier };
        let valid = block != block::AIR && (tier == 0 || tiers::item_of(block, tier).is_some());
        let free = ghost.cells().into_iter().all(|c| block::replaceable(self.world.block_anywhere_or_generate(c)));
        if valid && free {
            self.ghosts.set(ghost);
        }
    }
}

#[cfg(test)]
mod tests;
