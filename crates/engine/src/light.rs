//! Light for the chunk mesher: sky light and block light, 0..=15 each, computed when a chunk meshes.
//! Part of the render cache, derived from blocks only, never core state (DEV_PLAN 3.4).
//!
//! The field covers the chunk plus a [`MARGIN`] of 32 blocks on every side, read from the 3×3×3
//! neighbours the mesher already has, so light from anything that can reach the chunk is included
//! and borders match. Sky light is 15 in every cell open to the sky straight above (the columns above
//! the field come from the chunks higher up); it floods outward, losing 1 per block, through
//! everything that isn't an opaque cube. Leaves let flooded light through but stop the straight sky
//! column, so forests are shady; water does too and takes twice as much per block.
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

use crate::block::{BlockId, Render, BLOCK_COUNT, DEFS, GLASS, HOIST, LADDER};
use crate::chunk::{index, Chunk};
use crate::mesher::{neighbor_index, PAD, PAD_VOLUME};

/// Kinds of block light (`BlockDef::light` indexes this): strength at the block, loss per block.
pub const SOURCES: [(u8, u8); 3] = [(0, 0), (32, 1), (24, 2)];
pub const LAMP_LIGHT: u8 = 1;
pub const TORCH_LIGHT: u8 = 2;

/// How far past the chunk the field reaches: the farthest any light travels (a lamp's 32).
pub const MARGIN: usize = 32;
/// The field's size per axis (the chunk plus both margins), and with a wall of one cell around it.
const R: usize = 32 + 2 * MARGIN;
const W: usize = R + 2;
const WW: usize = W * W;
const FULL: u8 = 15;

/// Per block: bit 0 lets light through, bit 1 also lets the straight sky column through, bit 2 dims it
/// twice as fast; its `SOURCES` row in the high nibble.
const CLASS: [u8; 256] = {
    let mut t = [0u8; 256];
    let mut i = 0;
    while i < BLOCK_COUNT {
        let passes = match DEFS[i].render {
            Render::Opaque => 0,
            Render::Cutout if i != GLASS as usize && i != LADDER as usize && i != HOIST as usize => PASSES,
            Render::Liquid => PASSES | DIMS,
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

/// How far from a changed block the light can change when `old` becomes `new`, in blocks, and whether only
/// the sky light can: 0 when light treats the two alike (stone to dirt); a full sky light's reach, sky only,
/// when just the straight sky column differs (leaves to air); else the whole field margin (something lets
/// light through or shines).
pub fn reach(old: BlockId, new: BlockId) -> (i32, bool) {
    let (a, b) = (CLASS[old as usize], CLASS[new as usize]);
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

/// Scratch buffers, reused for every chunk.
pub struct Lighting {
    class: Vec<u8>,
    sky: Vec<u8>,
    block: Vec<u8>,
    /// Light from sources that lose 2 per block; all zero between chunks (reset cell by cell).
    short: Vec<u8>,
    queue: Vec<u32>,
    block_seeds: Vec<u32>,
    short_seeds: Vec<u32>,
    /// Per field column, the lowest field y open to the sky (`R + 1` when none is; 0 for the walls).
    floor: Vec<u16>,
    /// Per field column, whether something above the field shades it.
    shaded: Vec<bool>,
    /// The result in the mesher's padded layout: sky | block << 4.
    pub pad: Vec<u8>,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            class: vec![0; W * WW],
            sky: vec![0; W * WW],
            block: vec![0; W * WW],
            short: vec![0; W * WW],
            queue: Vec::new(),
            block_seeds: Vec::new(),
            short_seeds: Vec::new(),
            floor: vec![0; WW],
            shaded: vec![false; WW],
            pad: vec![0; PAD_VOLUME],
        }
    }
}

impl Lighting {
    /// Lights the centre chunk of `n` (see `mesher::neighbor_index`) into `pad`. `above[dx + 1 +
    /// (dz + 1) * 3]` lists the chunks above the neighbourhood in that chunk column.
    pub fn light(&mut self, n: &[&Chunk; 27], above: &[&[&Chunk]; 9]) {
        self.fill(n);
        self.seed_sources();
        self.shade_from_above(n, above);
        self.sky_columns();
        self.seed_sky();
        flood(&self.class, &mut self.sky, &mut self.queue, 1);
        flood(&self.class, &mut self.block, &mut self.block_seeds, 1);
        flood(&self.class, &mut self.short, &mut self.short_seeds, 2);
        self.write_pad();
        for &i in &self.short_seeds {
            self.short[i as usize] = 0;
        }
        self.queue.clear();
        self.block_seeds.clear();
        self.short_seeds.clear();
    }

    /// Copies block classes into the field (the walls around it stay 0) and seeds block light at
    /// emitting blocks.
    fn fill(&mut self, n: &[&Chunk; 27]) {
        self.block.fill(0);
        for wy in 1..=R {
            let (cy, ly) = split(wy);
            for wz in 1..=R {
                let (cz, lz) = split(wz);
                let row = (wy * W + wz) * W;
                for (cx, x0, lx0, len) in
                    [(0, 1, 32 - MARGIN, MARGIN), (1, 1 + MARGIN, 0, 32), (2, 1 + MARGIN + 32, 0, MARGIN)]
                {
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
        self.shaded.fill(false);
        for cz in 0..3 {
            for cx in 0..3 {
                let top = n[neighbor_index(cx as i32 - 1, 1, cz as i32 - 1)];
                let chunks = above[cx + cz * 3].iter().map(|&c| (c, 0)).chain([(top, MARGIN)]);
                for (chunk, lowest) in chunks {
                    if let Some(b) = chunk.as_uniform() {
                        if CLASS[b as usize] & SKY_PASSES == 0 {
                            for wz in columns(cz) {
                                self.shaded[wz * W + columns(cx).start..wz * W + columns(cx).end].fill(true);
                            }
                        }
                        continue;
                    }
                    let d = chunk.dense().expect("a chunk is uniform or dense");
                    for wz in columns(cz) {
                        let lz = split(wz).1;
                        for wx in columns(cx) {
                            let col = wz * W + wx;
                            let lx = split(wx).1;
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
        self.sky.fill(0);
        for wz in 1..=R {
            for wx in 1..=R {
                let col = wz * W + wx;
                let mut y = R + 1;
                if !self.shaded[col] {
                    while y > 1 && self.class[(y - 1) * WW + col] & SKY_PASSES != 0 {
                        y -= 1;
                        self.sky[y * WW + col] = FULL;
                    }
                }
                self.floor[col] = y as u16;
            }
        }
    }

    /// Queues the sky cells next to cells the sky doesn't reach: beside a column whose sky ends
    /// higher, and the lowest sky cell of each column (for leaves and plants below it).
    fn seed_sky(&mut self) {
        self.queue.clear();
        for wz in 1..=R {
            for wx in 1..=R {
                let col = wz * W + wx;
                let f = self.floor[col] as usize;
                let reach =
                    [col - 1, col + 1, col - W, col + W].iter().map(|&c| self.floor[c] as usize).max().unwrap_or(0);
                for y in f..reach {
                    self.queue.push((y * WW + col) as u32);
                }
                if f <= R && f > 1 && reach <= f {
                    self.queue.push((f * WW + col) as u32);
                }
            }
        }
    }

    /// Copies the chunk and its one-block border into `pad`.
    fn write_pad(&mut self) {
        for py in 0..PAD {
            for pz in 0..PAD {
                let from = ((py + MARGIN) * W + pz + MARGIN) * W + MARGIN;
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
/// through cells that let light pass. Leaves every cell it lit in `queue`.
fn flood(class: &[u8], light: &mut [u8], queue: &mut Vec<u32>, loss: u8) {
    let mut head = 0;
    while head < queue.len() {
        let i = queue[head] as usize;
        head += 1;
        let l = light[i];
        if l <= loss {
            continue;
        }
        for j in [i - 1, i + 1, i - W, i + W, i - WW, i + WW] {
            let to = l.saturating_sub(loss << ((class[j] & DIMS) / DIMS));
            if class[j] & PASSES != 0 && light[j] < to {
                light[j] = to;
                queue.push(j as u32);
            }
        }
    }
}

/// A field coordinate (1..=R) as (neighbour 0..3, chunk-local coordinate).
#[inline]
fn split(w: usize) -> (usize, usize) {
    let c = w - 1 + 32 - MARGIN;
    (c / 32, c % 32)
}

/// The field columns (1..=R) that fall in neighbour `c` (0..3) along one axis.
fn columns(c: usize) -> std::ops::Range<usize> {
    match c {
        0 => 1..1 + MARGIN,
        1 => 1 + MARGIN..1 + MARGIN + 32,
        _ => 1 + MARGIN + 32..R + 1,
    }
}

#[cfg(test)]
mod tests;
