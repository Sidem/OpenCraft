//! Light for the chunk mesher: sky light and block light, 0..=15 each, computed when a chunk meshes.
//! Part of the render cache, derived from blocks only, never core state (DEV_PLAN 3.4).
//!
//! The field covers the chunk plus a margin on every side, read from the 3×3×3 neighbours the mesher already has, so
//! light from anything that can reach the chunk is included and borders match: [`MARGIN`] (32 blocks) when any of
//! the 27 chunks holds a block that shines (`Chunk::holds`), else [`SKY_MARGIN`], enough for sky light,
//! which is gone 15 blocks out (the same result in a third of the cells). Sky light is 15 in every cell open to the
//! sky straight above (the columns above the field come from the chunks higher up); it floods outward, losing 1 per
//! block, through everything that isn't an opaque cube. Leaves let flooded light through but stop the straight sky
//! column, so forests are shady; so does a grown tree's wood (drawn round, so the ground beside it shows); water
//! does too and takes twice as much per block.
//!
//! Block light starts at emitting blocks (`BlockDef::light` names a row of [`SOURCES`]: strength and
//! loss per block). Sources losing 1 per block flood one field, those losing 2 (torches: bright but
//! short) another; a cell shows the larger, at most 15. A lamp (32) is at full brightness within 17
//! blocks and reaches 31; a torch (24) is within 4 and reaches 11. `MARGIN` can't exceed 32: the field is
//! read from the 3×3×3 neighbours.
//!
//! The result (`pad`) uses the mesher's padded layout (`mesher::pidx`): sky in the low nibble, block
//! light in the high one. To change how a block treats light: `CLASS` below; to add a kind of light:
//! a row in `SOURCES` (a loss of 1 or 2).

use crate::block::{is_tree_wood, BlockId, Render, BLOCK_COUNT, DEFS, GLASS, HOIST, LADDER, TABLE_LEN};
use crate::chunk::{index, Chunk, HOLDS_LIGHT};
use crate::mesher::{neighbor_index, PAD, PAD_VOLUME};

/// Kinds of block light (`BlockDef::light` indexes this): strength at the block, loss per block.
pub const SOURCES: [(u8, u8); 6] = [(0, 0), (32, 1), (24, 2), (40, 2), (52, 2), (64, 2)];
pub const LAMP_LIGHT: u8 = 1;
pub const TORCH_LIGHT: u8 = 2;
/// The LED lamp's three steps (`block/led.rs`): full within 12, 18 and 24 blocks, none past 20, 26 and 32. All lose 2 a
/// block, so they share the torches' field, and none reaches `MARGIN` with light (a loss of 1 would seam at chunk borders).
pub const LED_LIGHTS: [u8; 3] = [3, 4, 5];

/// How far past the chunk the field reaches when something shines: the farthest any light travels (a lamp's 32).
pub const MARGIN: usize = 32;
/// The margin with nothing shining: sky light from 15 is 0 by the time it is 15 blocks past the padded chunk.
const SKY_MARGIN: usize = 16;
/// The largest field's size per axis with a wall of one cell around it.
const MAX_W: usize = 32 + 2 * MARGIN + 2;
const FULL: u8 = 15;

/// Per block: bit 0 lets light through, bit 1 also lets the straight sky column through, bit 2 dims it
/// twice as fast; its `SOURCES` row in the high nibble.
const CLASS: [u8; TABLE_LEN] = {
    let mut t = [0u8; TABLE_LEN];
    let mut i = 0;
    while i < BLOCK_COUNT {
        let passes = match DEFS[i].render {
            // A grown tree's wood is drawn round and thinner than its cell: the ground beside it shows, lit.
            _ if is_tree_wood(i as BlockId) => PASSES,
            Render::Opaque => 0,
            Render::Cutout if i != GLASS as usize && i != LADDER as usize && i != HOIST as usize => PASSES,
            Render::Liquid => PASSES | DIMS,
            Render::Translucent => PASSES | SKY_PASSES,
            _ => PASSES | SKY_PASSES,
        };
        t[i] = passes | (DEFS[i].light << 4);
        i += 1;
    }
    t
};
const PASSES: u8 = 1;
const SKY_PASSES: u8 = 2;
/// Light loses 2 per block through it instead of 1 (water).
const DIMS: u8 = 4;

/// Light class of `b` with shape modifier `shape`: any non-full shape lets flooded light pass around it
/// while keeping its sky-column rule (so pitched roofs still shade rooms below, and stairs are never pitch black).
#[inline]
fn class_of(b: BlockId, shape: u8) -> u8 {
    let c = CLASS[b as usize];
    if shape != 0 {
        c | PASSES
    } else {
        c
    }
}

/// How far from a changed block the light can change when `(old, old_shape)` becomes `(new, new_shape)`,
/// in blocks, and whether only the sky light can: 0 when light treats the two alike (stone to dirt); a full
/// sky light's reach, sky only, when just the straight sky column differs (leaves to air); else the whole
/// field margin (something lets light through or shines).
pub fn reach_shaped(old: BlockId, old_shape: u8, new: BlockId, new_shape: u8) -> (i32, bool) {
    let (a, b) = (class_of(old, old_shape), class_of(new, new_shape));
    if a == b {
        (0, false)
    } else if (a ^ b) & !SKY_PASSES == 0 {
        (i32::from(FULL), true)
    } else {
        (MARGIN as i32, false)
    }
}

/// Whether the straight sky column goes through `b`.
pub fn passes_sky(b: BlockId) -> bool {
    CLASS[b as usize] & SKY_PASSES != 0
}

/// Scratch buffers, reused for every chunk, sized for the largest field.
pub struct Lighting {
    /// The current field's margin, and its size per axis with the walls (`w`); the cells outside the current field
    /// may hold anything.
    m: usize,
    w: usize,
    class: Vec<u8>,
    sky: Vec<u8>,
    /// Light from sources that lose 1 per block, and from those that lose 2; both all zero between chunks (reset cell
    /// by cell from the flood queues).
    block: Vec<u8>,
    short: Vec<u8>,
    queue: Vec<u32>,
    block_seeds: Vec<u32>,
    short_seeds: Vec<u32>,
    /// Per field column, the lowest field y open to the sky (`r + 1` when none is; 0 for the walls).
    floor: Vec<u16>,
    /// Per field column, whether something above the field shades it.
    shaded: Vec<bool>,
    /// The result in the mesher's padded layout: sky | block << 4.
    pub pad: Vec<u8>,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            m: 0,
            w: 0,
            class: vec![0; MAX_W * MAX_W * MAX_W],
            sky: vec![0; MAX_W * MAX_W * MAX_W],
            block: vec![0; MAX_W * MAX_W * MAX_W],
            short: vec![0; MAX_W * MAX_W * MAX_W],
            queue: Vec::new(),
            block_seeds: Vec::new(),
            short_seeds: Vec::new(),
            floor: vec![0; MAX_W * MAX_W],
            shaded: vec![false; MAX_W * MAX_W],
            pad: vec![0; PAD_VOLUME],
        }
    }
}

impl Lighting {
    /// Lights the centre chunk of `n` (see `mesher::neighbor_index`) into `pad`. `above[dx + 1 +
    /// (dz + 1) * 3]` lists the chunks above the neighbourhood in that chunk column.
    pub fn light(&mut self, n: &[&Chunk; 27], above: &[&[&Chunk]; 9]) {
        let m = if n.iter().any(|c| c.holds(HOLDS_LIGHT)) { MARGIN } else { SKY_MARGIN };
        if m != self.m {
            self.m = m;
            self.w = 32 + 2 * m + 2;
            self.clear_walls();
        }
        let w = self.w;
        self.fill(n);
        self.seed_sources();
        self.shade_from_above(n, above);
        self.sky_columns();
        self.seed_sky();
        flood(&self.class, &mut self.sky, &mut self.queue, 1, w);
        flood(&self.class, &mut self.block, &mut self.block_seeds, 1, w);
        flood(&self.class, &mut self.short, &mut self.short_seeds, 2, w);
        self.write_pad();
        for &i in &self.block_seeds {
            self.block[i as usize] = 0;
        }
        for &i in &self.short_seeds {
            self.short[i as usize] = 0;
        }
        self.queue.clear();
        self.block_seeds.clear();
        self.short_seeds.clear();
    }

    /// The current field's size per axis without the walls.
    fn r(&self) -> usize {
        self.w - 2
    }

    /// Zeroes the class of the current field's walls (`fill` never writes them; a field of the other size may have).
    fn clear_walls(&mut self) {
        let w = self.w;
        let at = |y: usize, z: usize, x: usize| (y * w + z) * w + x;
        for a in 0..w {
            for b in 0..w {
                for i in [at(0, a, b), at(w - 1, a, b), at(a, 0, b), at(a, w - 1, b), at(a, b, 0), at(a, b, w - 1)] {
                    self.class[i] = 0;
                }
            }
        }
    }

    /// Copies block classes into the field (the walls around it stay 0) and seeds block light at
    /// emitting blocks.
    fn fill(&mut self, n: &[&Chunk; 27]) {
        let (m, r, w) = (self.m, self.r(), self.w);
        for wy in 1..=r {
            let (cy, ly) = split(wy, m);
            for wz in 1..=r {
                let (cz, lz) = split(wz, m);
                let row = (wy * w + wz) * w;
                for (cx, x0, lx0, len) in [(0, 1, 32 - m, m), (1, 1 + m, 0, 32), (2, 1 + m + 32, 0, m)] {
                    let chunk = n[neighbor_index(cx - 1, cy as i32 - 1, cz as i32 - 1)];
                    let start = row + x0;
                    let out = &mut self.class[start..start + len];
                    match chunk.dense() {
                        Some(d) => {
                            let s = index(lx0, ly, lz);
                            for (k, (o, &b)) in out.iter_mut().zip(&d[s..s + len]).enumerate() {
                                let c = CLASS[b as usize];
                                *o = c;
                                if c >= 16 {
                                    self.block_seeds.push((start + k) as u32);
                                }
                            }
                        }
                        None => {
                            let c = CLASS[chunk.get(0, 0, 0) as usize];
                            out.fill(c);
                            if c >= 16 {
                                self.block_seeds.extend(start as u32..(start + len) as u32);
                            }
                        }
                    }
                }
            }
        }
        // Any cell carrying a non-full shape modifier lets flooded light pass around it (zero cost for unshaped chunks).
        let base = 1isize + m as isize - 32;
        let inside = |v: isize| (1..=r as isize).contains(&v);
        for cz in 0..3isize {
            for cy in 0..3isize {
                for cx in 0..3isize {
                    let chunk = n[neighbor_index(cx as i32 - 1, cy as i32 - 1, cz as i32 - 1)];
                    if !chunk.has_shapes() {
                        continue;
                    }
                    for &(idx, _) in chunk.shapes() {
                        let lx = (idx & 31) as isize;
                        let lz = ((idx >> 5) & 31) as isize;
                        let ly = (idx >> 10) as isize;
                        let (wx, wy, wz) = (cx * 32 + lx + base, cy * 32 + ly + base, cz * 32 + lz + base);
                        if inside(wx) && inside(wy) && inside(wz) {
                            self.class[(wy as usize * w + wz as usize) * w + wx as usize] |= PASSES;
                        }
                    }
                }
            }
        }
    }

    /// Gives each emitter found by `fill` its strength, in the field its loss per block floods.
    fn seed_sources(&mut self) {
        let (class, block, short, short_seeds) = (&self.class, &mut self.block, &mut self.short, &mut self.short_seeds);
        self.block_seeds.retain(|&i| {
            let (strength, loss) = SOURCES[(class[i as usize] >> 4) as usize % SOURCES.len()];
            if loss == 1 {
                block[i as usize] = strength;
                return true;
            }
            short[i as usize] = strength;
            short_seeds.push(i);
            false
        });
    }

    /// Marks the field columns that something above the field shades: the chunks above the
    /// neighbourhood, and the part of the top neighbours above the field.
    fn shade_from_above(&mut self, n: &[&Chunk; 27], above: &[&[&Chunk]; 9]) {
        let (m, r, w) = (self.m, self.r(), self.w);
        self.shaded[..w * w].fill(false);
        for cz in 0..3 {
            for cx in 0..3 {
                let top = n[neighbor_index(cx as i32 - 1, 1, cz as i32 - 1)];
                let chunks = above[cx + cz * 3].iter().map(|&c| (c, 0)).chain([(top, m)]);
                for (chunk, lowest) in chunks {
                    if let Some(b) = chunk.as_uniform() {
                        if CLASS[b as usize] & SKY_PASSES == 0 {
                            let xs = columns(cx, m, r);
                            for wz in columns(cz, m, r) {
                                self.shaded[wz * w + xs.start..wz * w + xs.end].fill(true);
                            }
                        }
                        continue;
                    }
                    let d = chunk.dense().expect("a chunk is uniform or dense");
                    for wz in columns(cz, m, r) {
                        let lz = split(wz, m).1;
                        for wx in columns(cx, m, r) {
                            let col = wz * w + wx;
                            let lx = split(wx, m).1;
                            if !self.shaded[col] {
                                self.shaded[col] =
                                    (lowest..32).any(|y| CLASS[d[index(lx, y, lz)] as usize] & SKY_PASSES == 0);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Sets sky light 15 from the top of each unshaded column down to the first block that stops it.
    fn sky_columns(&mut self) {
        let (r, w) = (self.r(), self.w);
        let ww = w * w;
        self.sky[..w * ww].fill(0);
        for wz in 1..=r {
            for wx in 1..=r {
                let col = wz * w + wx;
                let mut y = r + 1;
                if !self.shaded[col] {
                    while y > 1 && self.class[(y - 1) * ww + col] & SKY_PASSES != 0 {
                        y -= 1;
                        self.sky[y * ww + col] = FULL;
                    }
                }
                self.floor[col] = y as u16;
            }
        }
        // The wall columns: no sky.
        for i in 0..w {
            for col in [i, (w - 1) * w + i, i * w, i * w + w - 1] {
                self.floor[col] = 0;
            }
        }
    }

    /// Queues the sky cells next to cells the sky doesn't reach: beside a column whose sky ends
    /// higher, and the lowest sky cell of each column (for leaves and plants below it).
    fn seed_sky(&mut self) {
        let (r, w) = (self.r(), self.w);
        let ww = w * w;
        self.queue.clear();
        for wz in 1..=r {
            for wx in 1..=r {
                let col = wz * w + wx;
                let f = self.floor[col] as usize;
                let reach =
                    [col - 1, col + 1, col - w, col + w].iter().map(|&c| self.floor[c] as usize).max().unwrap_or(0);
                for y in f..reach {
                    self.queue.push((y * ww + col) as u32);
                }
                if f <= r && f > 1 && reach <= f {
                    self.queue.push((f * ww + col) as u32);
                }
            }
        }
    }

    /// Copies the chunk and its one-block border into `pad`.
    fn write_pad(&mut self) {
        let (m, w) = (self.m, self.w);
        for py in 0..PAD {
            for pz in 0..PAD {
                let from = ((py + m) * w + pz + m) * w + m;
                let to = (py * PAD + pz) * PAD;
                for px in 0..PAD {
                    let block = self.block[from + px].max(self.short[from + px]).min(FULL);
                    self.pad[to + px] = self.sky[from + px] | (block << 4);
                }
            }
        }
    }
}

/// Floods light outward from the queued cells, `loss` less per step (twice that into cells that dim),
/// through cells that let light pass, in a field `w` cells wide. Leaves every cell it lit in `queue`.
fn flood(class: &[u8], light: &mut [u8], queue: &mut Vec<u32>, loss: u8, w: usize) {
    let ww = w * w;
    let mut head = 0;
    while head < queue.len() {
        let i = queue[head] as usize;
        head += 1;
        let l = light[i];
        if l <= loss {
            continue;
        }
        for j in [i - 1, i + 1, i - w, i + w, i - ww, i + ww] {
            let to = l.saturating_sub(loss << ((class[j] & DIMS) / DIMS));
            if class[j] & PASSES != 0 && light[j] < to {
                light[j] = to;
                queue.push(j as u32);
            }
        }
    }
}

/// A field coordinate (1..=r) with margin `m` as (neighbour 0..3, chunk-local coordinate).
#[inline]
fn split(c: usize, m: usize) -> (usize, usize) {
    let c = c - 1 + 32 - m;
    (c / 32, c % 32)
}

/// The field columns (1..=r) with margin `m` that fall in neighbour `c` (0..3) along one axis.
fn columns(c: usize, m: usize, r: usize) -> std::ops::Range<usize> {
    match c {
        0 => 1..1 + m,
        1 => 1 + m..1 + m + 32,
        _ => 1 + m + 32..r + 1,
    }
}

#[cfg(test)]
mod tests;
