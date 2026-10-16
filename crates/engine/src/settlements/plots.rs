//! Where a town's buildings go (step 13.3a): when a stage is built, each of its buildings takes the nearest free plot
//! slot round the depot that fits it. A plot fits when its ground varies at most [`MAX_SLOPE`] blocks, any water on it
//! is at most [`MAX_WATER`] deep, and from the ground up to the building's top it holds nothing but air, water, leaves,
//! logs and plants (so the player's blocks are never touched and nothing is dug). The building's floor is one above
//! the plot's highest ground or water; the foundation fills down to the ground, on stilts over water.
//!
//! Invariants: reads the world only through `block_anywhere_or_generate` and the generator, inside `Sim::step`, so the
//! choice is the same on every machine and whether chunks are loaded. A slot is used at most once per town.

use crate::block::{self, BlockId, LEAVES, LIQUID, LOG, PLANT, SAPLING};
use crate::bytes::{ByteReader, ByteWriter};
use crate::math::IVec3;
use crate::sim::Sim;
use crate::worldgen::SEA_LEVEL;

use super::plans::{cells, height, slot_offset, slots, Role, Shape, BUILDINGS, PALETTES};
use super::STAGES;

/// Most a plot's ground may rise across it.
pub const MAX_SLOPE: i32 = 3;
/// Deepest water a plot may stand in (on stilts).
pub const MAX_WATER: i32 = 3;

/// A building's place: its slot round the depot, which building (stage and index in `BUILDINGS`) and its floor.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Plot {
    pub slot: u8,
    pub stage: u8,
    pub building: u8,
    pub base: i32,
}

impl Plot {
    pub fn shape(&self) -> &'static Shape {
        &BUILDINGS[self.stage as usize - 1][self.building as usize]
    }

    /// Whether the building has a chimney in a town of `kind`.
    pub fn chimney(&self, kind: u8) -> bool {
        self.shape().chimney || PALETTES[kind as usize].chimneys
    }

    /// The building's lowest corner at floor level, for a town anchored at `anchor`.
    pub fn corner(&self, anchor: IVec3) -> IVec3 {
        let (s, mid) = (self.shape(), anchor + slot_offset(self.slot));
        IVec3::new(mid.x - s.w / 2, self.base, mid.z - s.d / 2)
    }

    /// Its cells in the world, bottom first, with their block and role, for a town of `kind` anchored at `anchor`.
    pub fn cells(&self, anchor: IVec3, kind: u8) -> Vec<(IVec3, BlockId, Role)> {
        let palette = &PALETTES[kind as usize];
        let corner = self.corner(anchor);
        let (i, j) = slots()[self.slot as usize];
        // The door faces the depot.
        let door = if i.abs() >= j.abs() { u8::from(i < 0) } else { 2 + u8::from(j < 0) };
        let cells = cells(self.shape(), palette.chimneys, door);
        cells.into_iter().map(|(at, role)| (corner + at, palette.block(role), role)).collect()
    }

    pub fn write(&self, w: &mut ByteWriter) {
        w.u8(self.slot);
        w.u8(self.stage);
        w.u8(self.building);
        w.i32(self.base);
    }

    /// Reads what `write` wrote; `None` on a slot, stage or building that doesn't exist.
    pub fn read(r: &mut ByteReader) -> Option<Plot> {
        let p = Plot { slot: r.u8()?, stage: r.u8()?, building: r.u8()?, base: r.i32()? };
        let fits = (p.slot as usize) < slots().len()
            && (1..=STAGES).contains(&(p.stage as usize))
            && (p.building as usize) < BUILDINGS[p.stage as usize - 1].len();
        fits.then_some(p)
    }
}

/// Whether a town may build over this block: air, water, leaves, logs and plants (a sapling too).
pub fn takeable(b: BlockId) -> bool {
    block::replaceable(b) || matches!(b, LOG | LEAVES | SAPLING) || PLANT[b as usize]
}

impl Sim {
    /// Plans town `i`'s next stage not yet planned: gives each of its buildings the nearest free slot that fits it (a
    /// building that fits nowhere is left out).
    pub(crate) fn plan_stage(&mut self, i: usize) {
        self.settlements.towns[i].planned += 1;
        let (anchor, built) = (self.settlements.towns[i].pos, self.settlements.towns[i].planned);
        let Some(shapes) = BUILDINGS.get(built as usize - 1) else { return };
        let mut used: Vec<u8> = self.settlements.towns[i].plots.iter().map(|p| p.slot).collect();
        for (b, s) in shapes.iter().enumerate() {
            let free = (0..slots().len() as u8).filter(|slot| !used.contains(slot)).collect::<Vec<_>>();
            for slot in free {
                if let Some(base) = self.plot_base(anchor + slot_offset(slot), s) {
                    self.settlements.towns[i].plots.push(Plot { slot, stage: built, building: b as u8, base });
                    used.push(slot);
                    break;
                }
            }
        }
    }

    /// The floor height of shape `s` centred on `mid`, or `None` when the plot doesn't fit it.
    fn plot_base(&mut self, mid: IVec3, s: &Shape) -> Option<i32> {
        let corner = IVec3::new(mid.x - s.w / 2, 0, mid.z - s.d / 2);
        let mut columns = Vec::with_capacity((s.w * s.d) as usize);
        for z in 0..s.d {
            for x in 0..s.w {
                columns.push((corner.x + x, corner.z + z, self.ground(corner.x + x, corner.z + z)?));
            }
        }
        let top = |c: &(i32, i32, (i32, i32))| c.2 .1;
        let (lo, hi) = (columns.iter().map(top).min()?, columns.iter().map(top).max()?);
        if hi - lo > MAX_SLOPE || columns.iter().any(|c| c.2 .1 - c.2 .0 > MAX_WATER) {
            return None;
        }
        let base = hi + 1;
        for &(x, z, (ground, _)) in &columns {
            for y in ground + 1..base + height(s) {
                if !takeable(self.world.block_anywhere_or_generate(IVec3::new(x, y, z))) {
                    return None;
                }
            }
        }
        Some(base)
    }

    /// The ground in column (x, z) and the top of the water on it (the ground again when dry), looking down past air,
    /// water, leaves, logs and plants; `None` when it isn't natural ground near the generator's height (a bridge,
    /// a pit).
    fn ground(&mut self, x: i32, z: i32) -> Option<(i32, i32)> {
        let h = self.world.generator().height_at(x, z);
        let mut water = None;
        for y in (h - 12..=h.max(SEA_LEVEL) + 16).rev() {
            let b = self.world.block_anywhere_or_generate(IVec3::new(x, y, z));
            if LIQUID[b as usize] {
                water.get_or_insert(y);
            } else if !takeable(b) {
                return (y <= h + 2).then_some((y, water.unwrap_or(y)));
            }
        }
        None
    }
}
