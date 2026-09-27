//! The maps' pictures: the minimap (a `MAP_SIZE`² RGBA image, one pixel per block column, north (-Z)
//! up, centred on the local player) and the world map (key M: any window of the explored world at 1 to
//! 16 blocks per pixel, `draw`). Presentation only: both read the explored map (`atlas.rs`), which is
//! built from loaded chunks (the render cache), never core state.
//!
//! A pixel is its column's top block colour (from the texture atlas; ore in its mark colour so exposed
//! ore stands out), lighter or darker by the height step to the column one pixel north-west; unseen
//! columns stay transparent. `redraw` does nothing unless the centre moved or the atlas changed; the host
//! limits how often it asks. To colour a block differently, change its texture, not this file.
//! Deposit, ore and machine marks over the images are in `minimap/marks.rs`.

mod atlas;
mod marks;

pub use atlas::Atlas;
pub use marks::{ore_color, Known, MARK_FIELDS};

use atlas::{column_index, Tile};

use crate::block::{is_ore, BlockId, AIR, BLOCK_COUNT, FACE_TEX};
use crate::chunk::CHUNK_SHIFT;
use crate::math::{IVec3, Vec3};
use crate::player::Player;
use crate::sim::PlayerId;
use crate::textures::TEX_SIZE;
use crate::world::World;

/// Width and height of the minimap in pixels (= blocks).
pub const MAP_SIZE: usize = 128;
/// The world map's largest image side, in pixels.
pub const WORLD_MAP_MAX: usize = 1024;
/// Brightness change per block of height step, in 1/256ths, and the step that saturates it.
const SHADE_PER_BLOCK: i32 = 32;
const SHADE_MAX_STEP: i32 = 4;

pub struct Minimap {
    /// Average top-face colour per block id.
    colors: [[u8; 3]; BLOCK_COUNT],
    /// Everywhere the local player has been.
    pub atlas: Atlas,
    centre: Option<(i32, i32)>,
    /// `atlas.changes` when the minimap was last drawn.
    drawn: u32,
    pub pixels: Vec<u8>,
    /// The world map's last image (`draw`).
    pub world_pixels: Vec<u8>,
    /// Deposits the local player has prospected (marks.rs).
    pub known: Known,
}

impl Minimap {
    pub fn new(textures: &[u8]) -> Self {
        let mut colors = [[0u8; 3]; BLOCK_COUNT];
        for (b, color) in colors.iter_mut().enumerate() {
            *color = if is_ore(b as BlockId) {
                ore_color(b as BlockId).to_be_bytes()[1..].try_into().unwrap()
            } else {
                average_color(textures, FACE_TEX[b][2] as usize)
            };
        }
        Self {
            colors,
            atlas: Atlas::default(),
            centre: None,
            drawn: 0,
            pixels: vec![0; MAP_SIZE * MAP_SIZE * 4],
            world_pixels: Vec::new(),
            known: Known::default(),
        }
    }

    /// A chunk meshed: the explored map picks up its column.
    pub fn touch(&mut self, chunk: IVec3) {
        self.atlas.touch(chunk.x, chunk.z);
    }

    /// Redraws the map around column `centre` if it moved or the explored map changed (rebuilding the
    /// changed tiles in range first). Returns whether the pixels changed.
    pub fn redraw(&mut self, world: &World, centre: (i32, i32)) -> bool {
        let (x0, z0) = (centre.0 - HALF, centre.1 - HALF);
        let last = MAP_SIZE as i32 - 1;
        let lo = ((x0 - 1) >> CHUNK_SHIFT, (z0 - 1) >> CHUNK_SHIFT);
        let hi = ((x0 + last) >> CHUNK_SHIFT, (z0 + last) >> CHUNK_SHIFT);
        self.atlas.refresh_in(world, lo, hi);
        if self.drawn == self.atlas.changes && self.centre == Some(centre) {
            return false;
        }
        self.centre = Some(centre);
        self.drawn = self.atlas.changes;
        let mut pixels = std::mem::take(&mut self.pixels);
        self.draw(x0, z0, 1, MAP_SIZE, MAP_SIZE, &mut pixels);
        self.pixels = pixels;
        true
    }

    /// Draws `w`×`h` pixels of the explored map into `out` (RGBA, resized to fit): pixel (i, j) shows
    /// column (x0 + i·scale, z0 + j·scale).
    pub fn draw(&self, x0: i32, z0: i32, scale: i32, w: usize, h: usize, out: &mut Vec<u8>) {
        out.resize(w * h * 4, 0);
        let (mut here, mut nw) = (Lookup::new(&self.atlas), Lookup::new(&self.atlas));
        for j in 0..h {
            let z = z0 + j as i32 * scale;
            for i in 0..w {
                let x = x0 + i as i32 * scale;
                let (v, n) = (here.get(x, z), nw.get(x - scale, z - scale));
                let p = (j * w + i) * 4;
                let block = (v & 0xff) as usize;
                if block == AIR as usize {
                    out[p..p + 4].fill(0);
                    continue;
                }
                let step = if n == 0 { 0 } else { (v >> 8) as i32 - (n >> 8) as i32 };
                let k = 256 + step.clamp(-SHADE_MAX_STEP, SHADE_MAX_STEP) * SHADE_PER_BLOCK;
                let [r, g, b] = self.colors[block].map(|c| ((c as i32 * k) >> 8).min(255) as u8);
                out[p..p + 4].copy_from_slice(&[r, g, b, 255]);
            }
        }
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

const HALF: i32 = MAP_SIZE as i32 / 2;

/// Reads columns from the atlas, remembering the last tile (neighbouring pixels nearly always share it).
struct Lookup<'a> {
    atlas: &'a Atlas,
    key: (i32, i32),
    tile: Option<&'a Tile>,
}

impl<'a> Lookup<'a> {
    fn new(atlas: &'a Atlas) -> Self {
        Self { atlas, key: (i32::MIN, i32::MIN), tile: None }
    }

    fn get(&mut self, x: i32, z: i32) -> u16 {
        let key = (x >> CHUNK_SHIFT, z >> CHUNK_SHIFT);
        if key != self.key {
            self.key = key;
            self.tile = self.atlas.tile(key.0, key.1);
        }
        self.tile.map_or(0, |t| t.columns[column_index(x, z)])
    }
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
