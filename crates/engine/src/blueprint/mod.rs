//! Blueprints: a copied arrangement of machines that can be stamped down again as ghosts (`ghosts.rs`).
//! The data and its bytes live here; the player's hands (selecting two corners, copying, holding, stamping,
//! the outlines and label) are `hands.rs`, and the wasm methods the library panel calls are
//! `api/blueprint.rs`. Blueprints are the player's own notes, like map pins: not core state and not in the
//! world save. The host keeps their bytes with the world's record in the browser (`export` / `import`).
//!
//! A blueprint holds every machine anchored inside the selected box: its offset from the box's lowest
//! corner, block, facing and tier. Plain terrain and building blocks are not copied (the engine cannot
//! tell a placed block from a natural one), and neither are machine settings (recipe, filter, rule): a
//! ghost carries none yet. Stamping turns it in quarter turns about the vertical axis, which turns
//! each machine's facing the same way, so multi-block machines keep their shape.
//!
//! Invariants: at most [`MAX_ENTRIES`] machines per blueprint, [`MAX_BLUEPRINTS`] per world, names of at
//! most [`MAX_NAME`] characters without control characters; the box is at most [`MAX_SPAN`] across.
//! To copy more of a machine: read it in `Factory::placed_as` and carry it in [`Entry`] (and `Ghost`).

mod hands;
#[cfg(test)]
mod tests;

use crate::block::{self, BlockId};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::tiers;
use crate::ghosts::{Ghost, Ghosts};
use crate::item::ItemId;
use crate::math::IVec3;
use crate::sim::Sim;

pub const MAX_ENTRIES: usize = 1024;
pub const MAX_BLUEPRINTS: usize = 64;
pub const MAX_NAME: usize = 32;
/// The most cells a copied box spans along an axis.
pub const MAX_SPAN: i32 = 48;
/// Bytes format version.
const FORMAT: u8 = 1;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Entry {
    /// From the box's lowest corner.
    pub off: IVec3,
    pub block: BlockId,
    pub facing: u8,
    pub tier: u8,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Blueprint {
    pub name: String,
    /// The box's size in cells.
    pub size: IVec3,
    pub entries: Vec<Entry>,
}

/// The player's blueprints and what the hands are doing with them (`hands.rs`).
#[derive(Default)]
pub struct Library {
    pub list: Vec<Blueprint>,
    /// The blueprint in hand (stamped by the use button in ghost mode) and its quarter turns.
    pub held: Option<usize>,
    pub turns: u8,
    /// The first corner marked, and the selected box (lowest and highest cell) once the second is.
    pub corner: Option<IVec3>,
    pub selection: Option<(IVec3, IVec3)>,
    /// What the last mark, copy or stamp did, for the label.
    pub note: String,
    /// Changes whenever the list does, so the host knows to save and redraw.
    pub version: u32,
}

impl Blueprint {
    /// Copies the machines anchored in the box between corners `a` and `b`.
    pub fn copy(sim: &Sim, a: IVec3, b: IVec3, name: String) -> Result<Blueprint, &'static str> {
        let lo = IVec3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
        let hi = IVec3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));
        let size = hi - lo + IVec3::new(1, 1, 1);
        if size.x > MAX_SPAN || size.y > MAX_SPAN || size.z > MAX_SPAN {
            return Err("That box is too big to copy");
        }
        let mut entries = Vec::new();
        for x in lo.x..=hi.x {
            for y in lo.y..=hi.y {
                for z in lo.z..=hi.z {
                    let cell = IVec3::new(x, y, z);
                    let Some((facing, tier)) = sim.factory.placed_as(cell) else { continue };
                    let Some(id) = sim.world.block_anywhere(cell) else { continue };
                    let tier = if tiers::family(id).is_some() { tier } else { 0 };
                    if entries.len() == MAX_ENTRIES {
                        return Err("That is too many machines for one blueprint");
                    }
                    entries.push(Entry { off: cell - lo, block: id, facing, tier });
                }
            }
        }
        if entries.is_empty() {
            return Err("No machines in that box");
        }
        Ok(Blueprint { name, size, entries })
    }

    /// The box size and entries after `turns` quarter turns (clockwise seen from above is facing + 1).
    pub fn turned(&self, turns: u8) -> (IVec3, Vec<Entry>) {
        let mut size = self.size;
        let mut entries = self.entries.clone();
        for _ in 0..turns % 4 {
            for e in &mut entries {
                e.off = IVec3::new(size.z - 1 - e.off.z, e.off.y, e.off.x);
                e.facing = (e.facing + 1) % 4;
            }
            size = IVec3::new(size.z, size.y, size.x);
        }
        (size, entries)
    }

    /// The ghosts that stamping it with its lowest corner at `origin` plants.
    pub fn ghosts_at(&self, origin: IVec3, turns: u8) -> Vec<Ghost> {
        let (_, entries) = self.turned(turns);
        entries.iter().map(|e| Ghost { pos: origin + e.off, block: e.block, facing: e.facing, tier: e.tier }).collect()
    }

    /// The items building every machine needs, most needed first.
    pub fn needs(&self) -> Vec<(ItemId, u32)> {
        let mut ghosts = Ghosts::default();
        // Anchors are distinct, so none replace another; the cap is the same as a blueprint's.
        for g in self.ghosts_at(IVec3::ZERO, 0) {
            ghosts.set(g);
        }
        ghosts.needs()
    }
}

/// The blueprints as bytes (what the host stores).
pub fn export(list: &[Blueprint]) -> Vec<u8> {
    let mut w = ByteWriter::default();
    w.u8(FORMAT);
    w.count(list.len());
    for b in list {
        w.count(b.name.len());
        b.name.bytes().for_each(|c| w.u8(c));
        w.ivec3(b.size);
        w.count(b.entries.len());
        for e in &b.entries {
            w.ivec3(e.off);
            w.u8(e.block);
            w.u8(e.facing);
            w.u8(e.tier);
        }
    }
    w.bytes
}

/// Reads [`export`]'s bytes; `None` for anything damaged or too large.
pub fn import(bytes: &[u8]) -> Option<Vec<Blueprint>> {
    let mut r = ByteReader::new(bytes);
    if r.u8()? != FORMAT {
        return None;
    }
    let n = r.count()?;
    if n > MAX_BLUEPRINTS {
        return None;
    }
    let mut list = Vec::new();
    for _ in 0..n {
        let len = r.count()?;
        if len > MAX_NAME * 4 {
            return None;
        }
        let name = String::from_utf8((0..len).map(|_| r.u8()).collect::<Option<Vec<u8>>>()?).ok()?;
        let size = r.ivec3()?;
        let count = r.count()?;
        if count > MAX_ENTRIES || size.x < 1 || size.y < 1 || size.z < 1 || size.x.max(size.y).max(size.z) > MAX_SPAN {
            return None;
        }
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let e = Entry { off: r.ivec3()?, block: r.block()?, facing: r.u8()?, tier: r.u8()? };
            let inside =
                (0..size.x).contains(&e.off.x) && (0..size.y).contains(&e.off.y) && (0..size.z).contains(&e.off.z);
            if !inside || e.facing >= 4 || e.block == block::AIR {
                return None;
            }
            entries.push(e);
        }
        list.push(Blueprint { name: clean_name(&name), size, entries });
    }
    Some(list)
}

/// A name the panel and label can show: no control characters, at most [`MAX_NAME`] characters.
pub fn clean_name(raw: &str) -> String {
    raw.chars().filter(|c| !c.is_control()).take(MAX_NAME).collect::<String>().trim().to_string()
}
