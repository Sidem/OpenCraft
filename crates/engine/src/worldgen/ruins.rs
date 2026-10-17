//! Ruins of the last factory (generator version 9, step R5a): rare sites of rusted machines, a broken belt run, girder
//! poles and a collapsed shaft down to spent rock, on a cracked concrete slab. Blocks are in `block/ruins.rs`.
//!
//! Where: the world is cut into 512×512 regions centred on the spawn, each with at most one ruin, a pure function of the
//! seed (`candidate`: a few hashed tries, the first on flat dry land wins; `WorldGen::ruin_at` caches the result). The
//! first ruin, the one in the spawn's region, lies 150 to 297 blocks from the spawn. A ruin is a list of blocks
//! (`build`), so a chunk stamps the ones that fall in it, like a tree, and generation stays order-independent. Trees
//! keep out of the slab (`WorldGen::ruin_covers`).
//! To add a part to a ruin: lay its blocks in `build`; keep every block within `RADIUS` of the centre.

use std::cell::RefCell;
use std::rc::Rc;

use rustc_hash::FxHashMap;

use super::{WorldGen, SEA_LEVEL};
use crate::block::{BlockId, AIR, CONCRETE, DIRT, LOGBOOK, RUSTED_GIRDER, RUSTED_MACHINE, RUSTED_PLATING, SPENT_ROCK};
use crate::chunk::{index, CHUNK_SIZE};
use crate::math::{hash3, IVec3};

/// A region's side. Region (0, 0) is centred on the spawn.
pub(super) const REGION: i32 = 512;
/// Candidates keep this far from their region's edge, so ruins of neighbouring regions never touch.
const MARGIN: i32 = 64;
/// How far from its centre a ruin's blocks reach (the slab and the shaft's wall).
pub(super) const RADIUS: i32 = 7;
/// The slab is a disc of this radius squared.
const SLAB_R2: i32 = 36;
/// Tries per region before it gets no ruin.
const ATTEMPTS: u32 = 8;
/// The most the ground may rise or fall across a site (blocks).
const MAX_RELIEF: i32 = 3;
/// The logbook's offset from the centre (x, z).
const LOGBOOK_AT: (i32, i32) = (2, -2);

/// The ruin of each region asked about, `None` for a region with none (`WorldGen::ruins`).
pub(super) type Cache = RefCell<FxHashMap<(i32, i32), Option<Rc<Ruin>>>>;

pub(super) struct Ruin {
    pub x: i32,
    pub z: i32,
    /// Where the ruin's logbook stands (`logbook.rs` reads it).
    pub logbook: IVec3,
    /// The lowest and highest block the ruin sets (`stamp` skips chunks outside).
    bottom: i32,
    top: i32,
    blocks: Vec<(IVec3, BlockId)>,
}

impl Ruin {
    /// Whether any of the ruin can fall in the column of chunks starting at (x0, z0).
    pub fn near_column(&self, x0: i32, z0: i32) -> bool {
        (x0 - RADIUS..x0 + CHUNK_SIZE + RADIUS).contains(&self.x)
            && (z0 - RADIUS..z0 + CHUNK_SIZE + RADIUS).contains(&self.z)
    }

    /// Whether the highest block set is above `y` (a column's `max_y` must reach it).
    pub fn top(&self) -> i32 {
        self.top
    }

    /// Sets the ruin's blocks that lie in the chunk at `base`.
    pub fn stamp(&self, base: IVec3, b: &mut [BlockId]) {
        if base.y > self.top || base.y + CHUNK_SIZE <= self.bottom {
            return;
        }
        for &(p, id) in &self.blocks {
            let l = p - base;
            if (0..CHUNK_SIZE).contains(&l.x) && (0..CHUNK_SIZE).contains(&l.y) && (0..CHUNK_SIZE).contains(&l.z) {
                b[index(l.x as usize, l.y as usize, l.z as usize)] = id;
            }
        }
    }

    /// Whether (x, z) is under the slab or close to it.
    pub fn covers(&self, x: i32, z: i32) -> bool {
        let (dx, dz) = (x - self.x, z - self.z);
        dx * dx + dz * dz <= (RADIUS + 1) * (RADIUS + 1)
    }
}

/// The region a world coordinate lies in.
pub(super) fn region_of(v: i32) -> i32 {
    (v + REGION / 2).div_euclid(REGION)
}

/// Try number `attempt` for the ruin of region (`rx`, `rz`): the spawn's region rings the spawn at 150 to 297 blocks,
/// every other region takes any spot at least `MARGIN` inside its edges.
fn candidate(seed: u32, rx: i32, rz: i32, attempt: u32) -> (i32, i32) {
    let h = hash3(seed ^ 0x2D1E, rx, attempt as i32, rz);
    if (rx, rz) == (0, 0) {
        let d = 150 + (h % 61) as i32;
        let t = ((h >> 8) % (2 * d as u32 + 1)) as i32 - d;
        return match h >> 30 {
            0 => (d, t),
            1 => (-d, t),
            2 => (t, d),
            _ => (t, -d),
        };
    }
    let g = hash3(seed ^ 0x2D1F, rx, attempt as i32, rz);
    let span = (REGION - 2 * MARGIN) as u32;
    (rx * REGION - REGION / 2 + MARGIN + (h % span) as i32, rz * REGION - REGION / 2 + MARGIN + (g % span) as i32)
}

impl WorldGen {
    /// The ruin of region (`rx`, `rz`), if it has one (cached).
    pub(super) fn ruin_at(&self, rx: i32, rz: i32) -> Option<Rc<Ruin>> {
        if let Some(r) = self.ruins.borrow().get(&(rx, rz)) {
            return r.clone();
        }
        let ruin = (0..ATTEMPTS).find_map(|a| self.try_site(rx, rz, a)).map(Rc::new);
        self.ruins.borrow_mut().insert((rx, rz), ruin.clone());
        ruin
    }

    /// The ruins whose blocks can fall in the column of chunks starting at (x0, z0) (none before version 9).
    pub(super) fn ruins_near(&self, x0: i32, z0: i32) -> Vec<Rc<Ruin>> {
        self.ruins_around(x0, z0).filter(|r| r.near_column(x0, z0)).collect()
    }

    /// Whether a ruin's slab or its surroundings cover (x, z): no tree grows there.
    pub(super) fn ruin_covers(&self, x: i32, z: i32) -> bool {
        self.ruins_around(x, z).any(|r| r.covers(x, z))
    }

    /// The ruins of the regions next to (x, z)'s: a ruin only ever reaches into the next region.
    pub(super) fn ruins_around(&self, x: i32, z: i32) -> impl Iterator<Item = Rc<Ruin>> + '_ {
        let (rx, rz) = (region_of(x), region_of(z));
        let near = self.version >= 9;
        (rz - 1..=rz + 1)
            .flat_map(move |j| (rx - 1..=rx + 1).map(move |i| (i, j)))
            .filter(move |_| near)
            .filter_map(|(i, j)| self.ruin_at(i, j))
    }

    /// A ruin for attempt `attempt` of the region if the land there is flat and dry.
    fn try_site(&self, rx: i32, rz: i32, attempt: u32) -> Option<Ruin> {
        let (x, z) = candidate(self.seed, rx, rz, attempt);
        let y = self.height_at(x, z);
        let (mut lo, mut hi) = (y, y);
        for dz in [-6, 0, 6] {
            for dx in [-6, 0, 6] {
                let (sx, sz) = (x + dx, z + dz);
                let h = self.height_at(sx, sz);
                if self.water_top(sx, sz, h) > h {
                    return None;
                }
                (lo, hi) = (lo.min(h), hi.max(h));
            }
        }
        if hi - lo > MAX_RELIEF || lo <= SEA_LEVEL + 1 {
            return None;
        }
        let biome = self.biome_at(x, z, y);
        let surface = Self::surface_v2(biome, y, self.slope_at(x, z)).0 .0;
        Some(build(self.seed, x, z, y, surface, |wx, wz| self.height_at(wx, wz)))
    }
}

/// The ruin centred on (`x`, `z`) with its slab at `y`: `ground` is the terrain height, `surface` the block on top of
/// it (shows through the slab's cracks).
fn build(seed: u32, x: i32, z: i32, y: i32, surface: BlockId, ground: impl Fn(i32, i32) -> i32) -> Ruin {
    let mut out = Vec::new();
    let mut put = |dx: i32, wy: i32, dz: i32, id: BlockId| out.push((IVec3::new(x + dx, wy, z + dz), id));
    let roll = |salt: i32, i: i32| hash3(seed ^ 0x2D20, x + salt, i, z - salt);

    // The slab: the ground cut or filled to the centre's height, then concrete with cracks (more at the rim).
    for dz in -RADIUS..=RADIUS {
        for dx in -RADIUS..=RADIUS {
            let d2 = dx * dx + dz * dz;
            if d2 > SLAB_R2 {
                continue;
            }
            let h = ground(x + dx, z + dz);
            (y + 1..=h).for_each(|wy| put(dx, wy, dz, AIR));
            (h + 1..y).for_each(|wy| put(dx, wy, dz, DIRT));
            let cracked = roll(1, dx * 64 + dz) % 100 < if d2 > 25 { 55 } else { 30 };
            put(dx, y, dz, if cracked { surface } else { CONCRETE });
        }
    }

    // Eight places for what stood on the slab: three machines, two girder poles, the others empty.
    const SLOTS: [(i32, i32); 8] = [(-4, -3), (3, -4), (-3, 3), (4, 2), (0, -5), (-5, 0), (5, -1), (1, 4)];
    let first = (roll(2, 0) % 8) as usize;
    let slot = |k: usize| SLOTS[(first + k) % 8];
    for (n, k) in [0usize, 3, 5].into_iter().enumerate() {
        let (dx, dz) = slot(k);
        put(dx, y + 1, dz, RUSTED_MACHINE);
        match roll(3, n as i32) % 6 {
            0 | 1 => put(dx, y + 2, dz, RUSTED_MACHINE),
            2 | 3 => put(dx, y + 2, dz, RUSTED_PLATING),
            _ => {}
        }
    }
    for (n, k) in [1usize, 6].into_iter().enumerate() {
        let (dx, dz) = slot(k);
        let height = 2 + (roll(4, n as i32) % 4) as i32;
        (1..=height).for_each(|up| put(dx, y + up, dz, RUSTED_GIRDER));
    }

    // A belt run: plating laid from the first machine towards the centre, with gaps where it rotted away.
    let (sx, sz) = slot(0);
    let along_x = sx.abs() >= sz.abs();
    for step in 1..=4 {
        let (dx, dz) = if along_x { (sx - sx.signum() * step, sz) } else { (sx, sz - sz.signum() * step) };
        let in_shaft = (0..=1).contains(&dx) && (0..=1).contains(&dz);
        if !in_shaft && dx * dx + dz * dz <= SLAB_R2 && roll(5, step) % 100 < 65 {
            put(dx, y + 1, dz, RUSTED_PLATING);
        }
    }

    // The shaft: a 2x2 hole at the centre, rubble in it, ending in a chamber of spent rock with one more husk.
    let depth = 9 + (roll(6, 0) % 5) as i32;
    for dz in -1..=2 {
        for dx in -1..=2 {
            let inner = (0..=1).contains(&dx) && (0..=1).contains(&dz);
            for wy in y - depth - 1..=y - depth + 2 {
                if !inner || wy == y - depth - 1 {
                    put(dx, wy, dz, SPENT_ROCK);
                }
            }
            if inner {
                (y - depth..=y).for_each(|wy| put(dx, wy, dz, AIR));
            } else if roll(7, dx * 8 + dz) % 2 == 0 {
                put(dx, y + 1, dz, RUSTED_PLATING);
            }
        }
    }
    for wy in y - depth + 1..y {
        let k = roll(8, wy);
        if k % 3 == 0 {
            put(((k >> 4) & 1) as i32, wy, ((k >> 5) & 1) as i32, RUSTED_PLATING);
        }
    }
    let k = roll(9, 0);
    put(((k >> 4) & 1) as i32, y - depth, ((k >> 5) & 1) as i32, RUSTED_MACHINE);

    // The logbook on the slab, in the one corner no part uses.
    put(LOGBOOK_AT.0, y + 1, LOGBOOK_AT.1, LOGBOOK);
    let logbook = IVec3::new(x + LOGBOOK_AT.0, y + 1, z + LOGBOOK_AT.1);

    let bottom = out.iter().map(|(p, _)| p.y).min().unwrap_or(y);
    let top = out.iter().filter(|(_, id)| *id != AIR).map(|(p, _)| p.y).max().unwrap_or(y);
    Ruin { x, z, logbook, bottom, top, blocks: out }
}

#[cfg(test)]
mod tests;
