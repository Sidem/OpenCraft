//! The minimap: a `MAP_SIZE`² RGBA image, one pixel per block column, north (-Z) up, centred on the
//! local player. Presentation only: it reads the loaded chunks (the render cache) and never core state.
//!
//! Each column's top block and height are cached per chunk column (`Tile`), built when first needed
//! and dropped when a chunk in that column meshes or unloads (`touch`), so a redraw only copies tiles
//! and shades them. A pixel is its top block's average top-face colour (from the texture atlas),
//! lighter or darker by the height step to its north-west neighbour; unknown columns stay transparent.
//! `redraw` does nothing unless the centre moved or a tile in range changed; the host limits how often
//! it asks. To colour a block differently, change its texture, not this file.

use crate::block::{AIR, BLOCK_COUNT, FACE_TEX};
use crate::chunk::{CHUNK_SHIFT, CHUNK_SIZE};
use crate::math::{IVec3, Vec3};
use crate::player::Player;
use crate::sim::PlayerId;
use crate::textures::TEX_SIZE;
use crate::world::World;
use crate::worldgen::WORLD_HEIGHT_CHUNKS;

/// Width and height of the map in pixels (= blocks).
pub const MAP_SIZE: usize = 128;
/// Brightness change per block of height step, in 1/256ths, and the step that saturates it.
const SHADE_PER_BLOCK: i32 = 32;
const SHADE_MAX_STEP: i32 = 4;

pub struct Minimap {
    /// Average top-face colour per block id.
    colors: [[u8; 3]; BLOCK_COUNT],
    /// Cached chunk columns near the centre; few enough (about 25) for a linear search.
    tiles: Vec<Tile>,
    centre: Option<(i32, i32)>,
    /// A tile in range changed since the last redraw.
    stale: bool,
    /// Packed columns of the map plus a border row and column on the north-west (scratch).
    grid: Vec<u16>,
    pub pixels: Vec<u8>,
}

impl Minimap {
    pub fn new(textures: &[u8]) -> Self {
        let mut colors = [[0u8; 3]; BLOCK_COUNT];
        for (b, color) in colors.iter_mut().enumerate() {
            *color = average_color(textures, FACE_TEX[b][2] as usize);
        }
        Self {
            colors,
            tiles: Vec::new(),
            centre: None,
            stale: true,
            grid: vec![0; (MAP_SIZE + 1) * (MAP_SIZE + 1)],
            pixels: vec![0; MAP_SIZE * MAP_SIZE * 4],
        }
    }

    /// A chunk meshed or unloaded: its column's tile is rebuilt when next needed.
    pub fn touch(&mut self, chunk: IVec3) {
        self.tiles.retain(|t| (t.cx, t.cz) != (chunk.x, chunk.z));
        if self.centre.is_some_and(|c| in_range(c, chunk.x, chunk.z)) {
            self.stale = true;
        }
    }

    /// Redraws the map around column `centre` if it moved or a tile in range changed. Returns whether
    /// the pixels changed.
    pub fn redraw(&mut self, world: &World, centre: (i32, i32)) -> bool {
        if !self.stale && self.centre == Some(centre) {
            return false;
        }
        self.centre = Some(centre);
        self.stale = false;
        self.tiles.retain(|t| in_range(centre, t.cx, t.cz));
        self.fill_grid(world, centre);
        self.shade();
        true
    }

    /// Other players' marks relative to the local player's position: (x, z offset in blocks, yaw) each.
    pub fn player_marks(bodies: &[Option<Player>], local: PlayerId, at: Vec3) -> Vec<f32> {
        let mut marks = Vec::new();
        for (i, body) in bodies.iter().enumerate() {
            let Some(b) = body.as_ref().filter(|_| i != local.0 as usize) else { continue };
            marks.extend_from_slice(&[(b.pos.x - at.x) as f32, (b.pos.z - at.z) as f32, b.yaw as f32]);
        }
        marks
    }
}

/// One chunk column's top blocks: `height << 8 | block` per column (index `z << 5 | x`), 0 = unknown.
struct Tile {
    cx: i32,
    cz: i32,
    columns: Box<[u16]>,
}

const HALF: i32 = MAP_SIZE as i32 / 2;
const GRID: usize = MAP_SIZE + 1;

/// The chunk columns the map (and its north-west border) covers around `centre`.
fn tile_span(centre: (i32, i32)) -> (i32, i32, i32, i32) {
    let (x0, z0) = (centre.0 - HALF - 1, centre.1 - HALF - 1);
    let (x1, z1) = (centre.0 + HALF - 1, centre.1 + HALF - 1);
    (x0 >> CHUNK_SHIFT, z0 >> CHUNK_SHIFT, x1 >> CHUNK_SHIFT, z1 >> CHUNK_SHIFT)
}

fn in_range(centre: (i32, i32), cx: i32, cz: i32) -> bool {
    let (tx0, tz0, tx1, tz1) = tile_span(centre);
    (tx0..=tx1).contains(&cx) && (tz0..=tz1).contains(&cz)
}

impl Minimap {
    /// Copies every tile's columns inside the window into `grid` (0 where nothing is loaded).
    fn fill_grid(&mut self, world: &World, centre: (i32, i32)) {
        self.grid.fill(0);
        let (gx0, gz0) = (centre.0 - HALF - 1, centre.1 - HALF - 1);
        let (tx0, tz0, tx1, tz1) = tile_span(centre);
        for cz in tz0..=tz1 {
            for cx in tx0..=tx1 {
                let i = match self.tiles.iter().position(|t| (t.cx, t.cz) == (cx, cz)) {
                    Some(i) => i,
                    None => {
                        let Some(columns) = build_tile(world, cx, cz) else { continue };
                        self.tiles.push(Tile { cx, cz, columns });
                        self.tiles.len() - 1
                    }
                };
                let tile = &self.tiles[i];
                let (bx, bz) = (cx * CHUNK_SIZE, cz * CHUNK_SIZE);
                for lz in 0..CHUNK_SIZE {
                    let gz = bz + lz - gz0;
                    if !(0..GRID as i32).contains(&gz) {
                        continue;
                    }
                    let lx0 = (gx0 - bx).max(0);
                    let lx1 = (gx0 + GRID as i32 - bx).min(CHUNK_SIZE);
                    for lx in lx0..lx1 {
                        let g = gz as usize * GRID + (bx + lx - gx0) as usize;
                        self.grid[g] = tile.columns[(lz << CHUNK_SHIFT | lx) as usize];
                    }
                }
            }
        }
    }

    /// Turns `grid` into pixels: block colour, shaded by the height step to the north-west.
    fn shade(&mut self) {
        for z in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                let (v, nw) = (self.grid[(z + 1) * GRID + x + 1], self.grid[z * GRID + x]);
                let p = (z * MAP_SIZE + x) * 4;
                let block = (v & 0xff) as usize;
                if block == AIR as usize {
                    self.pixels[p..p + 4].fill(0);
                    continue;
                }
                let step = if nw == 0 { 0 } else { (v >> 8) as i32 - (nw >> 8) as i32 };
                let k = 256 + step.clamp(-SHADE_MAX_STEP, SHADE_MAX_STEP) * SHADE_PER_BLOCK;
                let [r, g, b] = self.colors[block].map(|c| ((c as i32 * k) >> 8).min(255) as u8);
                self.pixels[p..p + 4].copy_from_slice(&[r, g, b, 255]);
            }
        }
    }
}

/// The top non-air block of each column in chunk column (`cx`, `cz`), from its loaded chunks (an
/// unloaded chunk counts as air). `None` if none of them is loaded.
fn build_tile(world: &World, cx: i32, cz: i32) -> Option<Box<[u16]>> {
    const COLUMNS: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize;
    let mut columns = vec![0u16; COLUMNS];
    let (mut any, mut left) = (false, COLUMNS);
    for cy in (0..WORLD_HEIGHT_CHUNKS).rev() {
        let Some(chunk) = world.loaded_chunk(IVec3::new(cx, cy, cz)) else { continue };
        any = true;
        let base = (cy * CHUNK_SIZE) as u16;
        match (chunk.as_uniform(), chunk.dense()) {
            (Some(AIR), _) => continue,
            (Some(b), _) => {
                let top = (base + CHUNK_SIZE as u16 - 1) << 8 | b as u16;
                columns.iter_mut().filter(|c| **c == 0).for_each(|c| *c = top);
                left = 0;
            }
            (None, Some(blocks)) => {
                for (i, c) in columns.iter_mut().enumerate().filter(|(_, c)| **c == 0) {
                    let top = (0..CHUNK_SIZE as usize).rev().find(|&y| blocks[y << 10 | i] != AIR);
                    if let Some(y) = top {
                        *c = (base + y as u16) << 8 | blocks[y << 10 | i] as u16;
                        left -= 1;
                    }
                }
            }
            (None, None) => unreachable!("a chunk is uniform or dense"),
        }
        if left == 0 {
            break;
        }
    }
    any.then(|| columns.into_boxed_slice())
}

/// Average colour of a texture layer's visible pixels.
fn average_color(textures: &[u8], layer: usize) -> [u8; 3] {
    let texels = TEX_SIZE * TEX_SIZE;
    let (mut sum, mut n) = ([0u32; 3], 0u32);
    for px in textures[layer * texels * 4..(layer + 1) * texels * 4].chunks_exact(4) {
        if px[3] > 0 {
            (0..3).for_each(|c| sum[c] += px[c] as u32);
            n += 1;
        }
    }
    sum.map(|s| (s / n.max(1)) as u8)
}

#[cfg(test)]
mod tests;
