//! A quarry's dig box: which cells it digs and in what order (`DigBox`), the size and depth choices
//! (`WIDTHS`, `DEPTHS`; their indices are saved), what counts as ground (`QUARRIABLE`), and `survey`
//! (what is left to dig, for the panel and the placing preview). Geometry only: no state.
//! To offer another size or depth: append to `WIDTHS` or `DEPTHS` (never reorder).

use crate::block;
use crate::item::ItemId;
use crate::math::{sort_small_by_key, IVec3};
use crate::world::World;
use crate::worldgen::SEA_LEVEL;

use super::super::DIRS;

/// The box's widths and depths the panel offers (indices are saved); a new quarry takes the defaults.
pub const WIDTHS: [u8; 4] = [5, 7, 9, 11];
pub const DEPTHS: [Depth; 4] = [Depth::Layers(8), Depth::Layers(16), Depth::SeaLevel, Depth::Bedrock];
pub const DEFAULT_WIDTH: u8 = 1;
pub const DEFAULT_DEPTH: u8 = 1;

/// What the quarry digs: stone, the rocks, soil and sand. Ore, spent rock (part of a deposit),
/// bedrock, wood, glass, lamps and machines stay.
pub const QUARRIABLE: [bool; 256] = {
    let mut t = [false; 256];
    let ids = [
        block::STONE,
        block::DIRT,
        block::GRASS,
        block::SAND,
        block::GRANITE,
        block::SANDSTONE,
        block::BASALT,
        block::RUSTY_SOIL,
        block::DARK_SOIL,
        block::GREEN_SOIL,
        block::PALE_SOIL,
        block::RUSTY_SAND,
        block::DARK_SAND,
        block::GREEN_SAND,
        block::PALE_SAND,
    ];
    let mut i = 0;
    while i < ids.len() {
        t[ids[i] as usize] = true;
        i += 1;
    }
    t
};

/// How deep the box goes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Depth {
    /// This many layers below the quarry's own level.
    Layers(u8),
    /// Down to the layer just above the sea, so the pit never floods from it.
    SeaLevel,
    /// Down to the bedrock floor at y 0.
    Bedrock,
}

impl Depth {
    pub fn label(self) -> String {
        match self {
            Depth::Layers(n) => format!("{n} deep"),
            Depth::SeaLevel => "to sea level".to_string(),
            Depth::Bedrock => "to bedrock".to_string(),
        }
    }
}

/// The cells a quarry digs, in order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DigBox {
    pub pos: IVec3,
    pub facing: u8,
    pub width: i32,
    /// The highest and lowest layer (`bottom > top` when there is nothing to dig).
    pub top: i32,
    pub bottom: i32,
}

impl DigBox {
    /// The box of a quarry at `pos` facing `facing` (a `DIRS` index) with the given choices.
    pub fn new(pos: IVec3, facing: u8, width: u8, depth: u8) -> DigBox {
        let bottom = match DEPTHS[depth as usize % DEPTHS.len()] {
            Depth::Layers(n) => pos.y - n as i32,
            Depth::SeaLevel => SEA_LEVEL + 1,
            Depth::Bedrock => 1,
        };
        let width = WIDTHS[width as usize % WIDTHS.len()] as i32;
        DigBox { pos, facing: facing % 4, width, top: pos.y, bottom: bottom.max(1) }
    }

    pub fn layers(&self) -> u32 {
        (self.top - self.bottom + 1).max(0) as u32
    }

    pub fn cells(&self) -> u32 {
        self.layers() * (self.width * self.width) as u32
    }

    /// Cell `i`: layer by layer from the top; in a layer, row by row away from the quarry, back and forth.
    pub fn cell(&self, i: u32) -> IVec3 {
        let w = self.width as u32;
        let (layer, k) = (i / (w * w), i % (w * w));
        let (row, c) = (k / w, k % w);
        let col = if row % 2 == 1 { w - 1 - c } else { c };
        let (d, r) = (DIRS[self.facing as usize], DIRS[(self.facing as usize + 1) % 4]);
        let (ahead, side) = (row as i32 + 1, col as i32 - self.width / 2);
        IVec3::new(
            self.pos.x + d.x * ahead + r.x * side,
            self.top - layer as i32,
            self.pos.z + d.z * ahead + r.z * side,
        )
    }

    /// The lowest and highest corner cells.
    pub fn bounds(&self) -> (IVec3, IVec3) {
        let w = self.width * self.width;
        let (a, b) = (self.cell(0), self.cell((w - 1) as u32));
        let lo = IVec3::new(a.x.min(b.x), self.bottom, a.z.min(b.z));
        (lo, IVec3::new(a.x.max(b.x), self.top, a.z.max(b.z)))
    }
}

/// What is left to dig from cell `from` on, among the loaded cells of `world` (a query):
/// how many blocks and the two commonest drops.
#[inline(never)]
pub fn survey(dig: &DigBox, from: u32, world: &World) -> (u32, Vec<ItemId>) {
    let mut counts: Vec<(ItemId, u32)> = Vec::new();
    for i in from..dig.cells() {
        let Some(b) = world.get_block(dig.cell(i)).filter(|&b| QUARRIABLE[b as usize]) else { continue };
        let drop = ItemId::block(block::def(b).drop);
        match counts.iter_mut().find(|c| c.0 == drop) {
            Some(c) => c.1 += 1,
            None => counts.push((drop, 1)),
        }
    }
    let total = counts.iter().map(|c| c.1).sum();
    sort_small_by_key(&mut counts, |c| u32::MAX - c.1);
    (total, counts.iter().take(2).map(|c| c.0).collect())
}
