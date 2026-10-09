//! The explored map (presentation only): the top block and height of every block column the local
//! player has had loaded, kept per chunk column (`Tile`) after its chunks unload, so the minimap and the
//! world map (key M) show everywhere they have been. Never core state: it reads loaded chunks only.
//!
//! A chunk meshing marks its column dirty (`touch`); dirty tiles are rebuilt from the loaded chunks a
//! few per frame (`refresh_some`) or all at once inside the minimap (`refresh_in`). Unloading keeps the
//! last picture. The host keeps `export`'s bytes with the world's record in the browser, like the
//! prospected deposits. At most `MAX_TILES` are kept; the farthest from the newest go first.
//!
//! Each tile also lists where ore shows at its surface (`spots`, one per `SPOT_CELL`² blocks), which
//! both maps mark. To keep more per column: widen `Tile::columns` and bump `FORMAT`.

use rustc_hash::FxHashMap;

use crate::block::{is_ore, BlockId, AIR};
use crate::chunk::{CHUNK_SHIFT, CHUNK_SIZE};
use crate::math::IVec3;
use crate::world::World;
use crate::worldgen::WORLD_HEIGHT_CHUNKS;

/// The most chunk columns remembered (2 KB each).
pub const MAX_TILES: usize = 4096;
/// Dirty tiles rebuilt per frame outside the minimap's window.
const TILES_PER_FRAME: usize = 4;
/// Ore spots: at most one per square of this many blocks on a side.
const SPOT_CELL: usize = 8;
/// First byte of `export`; `import` ignores anything else.
const FORMAT: u8 = 1;
const COLUMNS: usize = (CHUNK_SIZE * CHUNK_SIZE) as usize;

/// One chunk column's top blocks.
pub struct Tile {
    /// `height << 8 | block` per block column (index `z << 5 | x`); 0 = not seen.
    pub columns: Box<[u16]>,
    /// Where ore shows at the top: (x, z within the tile, ore).
    pub spots: Vec<(u8, u8, BlockId)>,
}

#[derive(Default)]
pub struct Atlas {
    tiles: FxHashMap<(i32, i32), Tile>,
    dirty: Vec<(i32, i32)>,
    /// Counts tile rebuilds, so a map can tell when to redraw.
    pub changes: u32,
}

impl Atlas {
    /// A chunk of column (`cx`, `cz`) meshed: its tile is rebuilt soon.
    pub fn touch(&mut self, cx: i32, cz: i32) {
        if !self.dirty.contains(&(cx, cz)) {
            self.dirty.push((cx, cz));
        }
    }

    /// Rebuilds a few dirty tiles, oldest first. Call once per frame.
    pub fn refresh_some(&mut self, world: &World) {
        for _ in 0..TILES_PER_FRAME.min(self.dirty.len()) {
            let (cx, cz) = self.dirty.remove(0);
            self.rebuild(world, cx, cz);
        }
    }

    /// Rebuilds every dirty or missing tile inside chunk columns `lo..=hi` (a missing one only if
    /// loaded). Returns whether any changed.
    pub fn refresh_in(&mut self, world: &World, lo: (i32, i32), hi: (i32, i32)) -> bool {
        let inside = |&(cx, cz): &(i32, i32)| (lo.0..=hi.0).contains(&cx) && (lo.1..=hi.1).contains(&cz);
        let due: Vec<(i32, i32)> = self.dirty.iter().copied().filter(inside).collect();
        self.dirty.retain(|c| !inside(c));
        let before = self.changes;
        for cz in lo.1..=hi.1 {
            for cx in lo.0..=hi.0 {
                if (due.contains(&(cx, cz)) || !self.tiles.contains_key(&(cx, cz)))
                    && (self.tiles.contains_key(&(cx, cz)) || world.player_sees_column(cx, cz))
                {
                    self.rebuild(world, cx, cz);
                }
            }
        }
        self.changes != before
    }

    pub fn tile(&self, cx: i32, cz: i32) -> Option<&Tile> {
        self.tiles.get(&(cx, cz))
    }

    /// Calls `f` with every remembered tile in chunk columns `lo..=hi`, in no particular order.
    pub fn each_tile_in(&self, lo: (i32, i32), hi: (i32, i32), mut f: impl FnMut(i32, i32, &Tile)) {
        let area = (hi.0 - lo.0 + 1) as i64 * (hi.1 - lo.1 + 1) as i64;
        if area > self.tiles.len() as i64 {
            let inside = |cx: i32, cz: i32| (lo.0..=hi.0).contains(&cx) && (lo.1..=hi.1).contains(&cz);
            self.tiles.iter().filter(|(k, _)| inside(k.0, k.1)).for_each(|(k, t)| f(k.0, k.1, t));
            return;
        }
        for cz in lo.1..=hi.1 {
            for cx in lo.0..=hi.0 {
                if let Some(t) = self.tiles.get(&(cx, cz)) {
                    f(cx, cz, t);
                }
            }
        }
    }

    /// Makes block column (x, z) show `block` (tests: an explored map without a world).
    #[cfg(test)]
    pub fn plant(&mut self, x: i32, z: i32, block: BlockId) {
        let tile = self
            .tiles
            .entry((x >> CHUNK_SHIFT, z >> CHUNK_SHIFT))
            .or_insert_with(|| Tile { columns: vec![0; COLUMNS].into_boxed_slice(), spots: Vec::new() });
        tile.columns[column_index(x, z)] = 64 << 8 | block as u16;
    }

    /// `height << 8 | block` of block column (x, z), 0 if never seen.
    #[cfg(test)]
    pub fn column(&self, x: i32, z: i32) -> u16 {
        self.tile(x >> CHUNK_SHIFT, z >> CHUNK_SHIFT).map_or(0, |t| t.columns[column_index(x, z)])
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    /// Every remembered tile as bytes for the host to store: `FORMAT`, then per tile its column (two
    /// i32) and its 1024 columns (u16), little-endian.
    pub fn export(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(1 + self.tiles.len() * (8 + COLUMNS * 2));
        out.push(FORMAT);
        for (&(cx, cz), tile) in &self.tiles {
            out.extend_from_slice(&cx.to_le_bytes());
            out.extend_from_slice(&cz.to_le_bytes());
            tile.columns.iter().for_each(|c| out.extend_from_slice(&c.to_le_bytes()));
        }
        out
    }

    /// Adds the tiles in `export`'s bytes (a broken or unknown format adds nothing). Tiles already
    /// here stay: they are newer.
    pub fn import(&mut self, bytes: &[u8]) {
        let Some((&FORMAT, rest)) = bytes.split_first() else { return };
        for rec in rest.chunks_exact(8 + COLUMNS * 2) {
            let cx = i32::from_le_bytes(rec[0..4].try_into().unwrap());
            let cz = i32::from_le_bytes(rec[4..8].try_into().unwrap());
            if self.tiles.contains_key(&(cx, cz)) || self.tiles.len() >= MAX_TILES {
                continue;
            }
            let columns: Box<[u16]> = rec[8..].chunks_exact(2).map(|b| u16::from_le_bytes([b[0], b[1]])).collect();
            self.tiles.insert((cx, cz), Tile { spots: spots(&columns), columns });
        }
        self.changes = self.changes.wrapping_add(1);
    }
}

/// Index of block column (x, z) inside its tile.
pub fn column_index(x: i32, z: i32) -> usize {
    ((z & (CHUNK_SIZE - 1)) << CHUNK_SHIFT | (x & (CHUNK_SIZE - 1))) as usize
}

impl Atlas {
    /// Rebuilds one tile from the loaded chunks; keeps the old one if none of its chunks is loaded.
    fn rebuild(&mut self, world: &World, cx: i32, cz: i32) {
        let Some(columns) = build_tile(world, cx, cz) else { return };
        if self.tiles.len() >= MAX_TILES && !self.tiles.contains_key(&(cx, cz)) {
            let far = |&(x, z): &(i32, i32)| (x - cx).abs().max((z - cz).abs());
            if let Some(&key) = self.tiles.keys().max_by_key(|k| (far(k), **k)) {
                self.tiles.remove(&key);
            }
        }
        self.tiles.insert((cx, cz), Tile { spots: spots(&columns), columns });
        self.changes = self.changes.wrapping_add(1);
    }
}

/// The top non-air block of each column in chunk column (`cx`, `cz`), from its loaded chunks (an
/// unloaded chunk counts as air). `None` if none of them is loaded.
fn build_tile(world: &World, cx: i32, cz: i32) -> Option<Box<[u16]>> {
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

/// The first column showing ore in each `SPOT_CELL`² square of a tile.
fn spots(columns: &[u16]) -> Vec<(u8, u8, BlockId)> {
    let side = CHUNK_SIZE as usize;
    let mut out = Vec::new();
    for (sz, sx) in (0..side / SPOT_CELL).flat_map(|z| (0..side / SPOT_CELL).map(move |x| (z, x))) {
        let cell = (0..SPOT_CELL * SPOT_CELL).map(|i| (sx * SPOT_CELL + i % SPOT_CELL, sz * SPOT_CELL + i / SPOT_CELL));
        if let Some((x, z, b)) =
            cell.map(|(x, z)| (x, z, (columns[z * side + x] & 0xff) as BlockId)).find(|s| is_ore(s.2))
        {
            out.push((x as u8, z as u8, b));
        }
    }
    out
}
