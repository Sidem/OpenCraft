//! Light for the chunk mesher: sky light and block light, 0..=15 each, computed when a chunk meshes.
//! Part of the render cache, derived from blocks only, never core state (DEV_PLAN 3.4).
//!
//! The field covers the chunk plus a [`MARGIN`] of 15 blocks on every side, read from the 3×3×3
//! neighbours the mesher already has, so light from anything that can reach the chunk is included
//! and borders match. Sky light is 15 in every cell open to the sky straight above (the columns above
//! the field come from the chunks higher up); both kinds then flood outward, losing 1 per block,
//! through everything that isn't an opaque cube. Leaves let flooded light through but stop the
//! straight sky column, so forests are shady; water does too and dims light by 2 per block. Block light starts at emitting blocks (`BlockDef::light`).
//!
//! The result (`pad`) uses the mesher's padded layout (`mesher::pidx`): sky in the low nibble, block
//! light in the high one. To change how a block treats light: `CLASS` below.

use crate::block::{Render, BLOCK_COUNT, DEFS, GLASS};
use crate::chunk::{index, Chunk};
use crate::mesher::{neighbor_index, PAD, PAD_VOLUME};

/// How far past the chunk the field reaches: light fades out within 15 steps.
pub const MARGIN: usize = 15;
/// The field's size per axis (the chunk plus both margins), and with a wall of one cell around it.
const R: usize = 32 + 2 * MARGIN;
const W: usize = R + 2;
const WW: usize = W * W;
const FULL: u8 = 15;

/// Per block: bit 0 lets light through, bit 1 also lets the straight sky column through, bit 2 dims it
/// twice as fast; the block light it gives off in the high nibble.
const CLASS: [u8; 256] = {
    let mut t = [0u8; 256];
    let mut i = 0;
    while i < BLOCK_COUNT {
        let passes = match DEFS[i].render {
            Render::Opaque => 0,
            Render::Cutout if i != GLASS as usize => PASSES,
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

/// Scratch buffers, reused for every chunk.
pub struct Lighting {
    class: Vec<u8>,
    sky: Vec<u8>,
    block: Vec<u8>,
    queue: Vec<u32>,
    block_seeds: Vec<u32>,
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
            queue: Vec::new(),
            block_seeds: Vec::new(),
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
        self.shade_from_above(n, above);
        self.sky_columns();
        self.seed_sky();
        flood(&self.class, &mut self.sky, &mut self.queue);
        flood(&self.class, &mut self.block, &mut self.block_seeds);
        self.write_pad();
    }

    /// Copies block classes into the field (the walls around it stay 0) and seeds block light at
    /// emitting blocks.
    fn fill(&mut self, n: &[&Chunk; 27]) {
        self.block.fill(0);
        self.block_seeds.clear();
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
                                    self.block[start + k] = c >> 4;
                                    self.block_seeds.push((start + k) as u32);
                                }
                            }
                        }
                        None => {
                            let c = CLASS[chunk.get(0, 0, 0) as usize];
                            out.fill(c);
                            if c >= 16 {
                                for k in start..start + len {
                                    self.block[k] = c >> 4;
                                    self.block_seeds.push(k as u32);
                                }
                            }
                        }
                    }
                }
            }
        }
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
                    self.pad[to + px] = self.sky[from + px] | (self.block[from + px] << 4);
                }
            }
        }
    }
}

/// Floods light outward from the queued cells, 1 less per step, through cells that let light pass.
fn flood(class: &[u8], light: &mut [u8], queue: &mut Vec<u32>) {
    let mut head = 0;
    while head < queue.len() {
        let i = queue[head] as usize;
        head += 1;
        let l = light[i];
        if l <= 1 {
            continue;
        }
        for j in [i - 1, i + 1, i - W, i + W, i - WW, i + WW] {
            let to = l.saturating_sub(1 + (class[j] & DIMS) / DIMS);
            if class[j] & PASSES != 0 && light[j] < to {
                light[j] = to;
                queue.push(j as u32);
            }
        }
    }
    queue.clear();
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
