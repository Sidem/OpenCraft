//! What the player stands in, for the host's ambient sound (`web/src/audio/ambience.ts`): the biome, how much
//! sky is overhead, how open the ground is, how high or deep they are, and how much sea, fresh water, trees and
//! working machinery are near. Presentation only: read from the loaded blocks and the factory, never saved,
//! never part of the core state; the host polls it a few times a second (`Game::surroundings`).
//!
//! Layout of the floats `survey` returns (mirrored by `SURROUNDINGS` in `ambience.ts`): the `BIOME`..`DIN`
//! constants below. To add a reading: a constant, one line in `survey`, and its use in the host.

use crate::block::{self, AIR};
use crate::factory::Factory;
use crate::math::{IVec3, Vec3};
use crate::world::World;
use crate::worldgen::{SEA_LEVEL, WORLD_HEIGHT};

/// Indices into the floats. `BIOME` is the `Biome` variant's number (0 plains, 1 desert, 2 highlands, 3 lowlands,
/// 4 basalt fields); `SKY`, `OPEN`, `SEA`, `FRESH`, `TREES` and `DIN` run 0 to 1; `HEIGHT` is blocks above sea
/// level (negative below) and `DEPTH` blocks below the ground's surface (0 at or above it).
pub const BIOME: usize = 0;
pub const SKY: usize = 1;
pub const OPEN: usize = 2;
pub const HEIGHT: usize = 3;
pub const DEPTH: usize = 4;
pub const SEA: usize = 5;
pub const FRESH: usize = 6;
pub const TREES: usize = 7;
pub const DIN: usize = 8;
pub const FLOATS: usize = 9;

/// How far up the sky is looked for, and how far sideways a wall closes the ground in.
const SKY_REACH: i32 = 64;
const WALL_REACH: i32 = 8;
/// Leaves overhead each dim the sky by this much, up to `MAX_LEAVES` of them.
const LEAF_SHADE: f32 = 0.25;
const MAX_LEAVES: i32 = 3;
/// Fresh water (water above sea level) and trees are looked for in a grid this far around (blocks) at these
/// strides; the sea, which carries further, in a wider and coarser one.
const NEAR: i32 = 12;
const NEAR_STRIDE: usize = 3;
const SEA_NEAR: i32 = 24;
const SEA_STRIDE: usize = 4;
/// Vertical sampling of the near grid: from this far below the eye to this far above it.
const BELOW: i32 = -8;
const ABOVE: i32 = 10;
/// Weighted hits (`1 - distance / reach` each) that count as "all around you" for each reading.
const SEA_FULL: f32 = 20.0;
const FRESH_FULL: f32 = 5.0;
const TREES_FULL: f32 = 14.0;
const DIN_FULL: f32 = 3.0;
const DIN_RADIUS: f64 = 28.0;

/// Reads the surroundings of `eye`. Blocks that are not loaded count as air.
pub fn survey(world: &World, factory: &Factory, eye: Vec3) -> [f32; FLOATS] {
    let cell = eye.floor();
    let at = |x: i32, y: i32, z: i32| world.get_block(IVec3::new(x, y, z)).unwrap_or(AIR);
    let mut out = [0.0; FLOATS];

    let gen = world.generator();
    let ground = gen.height_at(cell.x, cell.z);
    if gen.version() >= 2 {
        out[BIOME] = gen.biome_at(cell.x, cell.z, ground) as u8 as f32;
    }
    out[HEIGHT] = (eye.y - f64::from(SEA_LEVEL)) as f32;
    out[DEPTH] = (f64::from(ground) - eye.y).max(0.0) as f32;

    // Sky: stone or any opaque block overhead closes it; leaves dim it.
    let mut leaves = 0;
    out[SKY] = 1.0;
    for y in cell.y + 1..(cell.y + SKY_REACH).min(WORLD_HEIGHT) {
        let b = at(cell.x, y, cell.z);
        if block::is_leaves(b) {
            leaves = (leaves + 1).min(MAX_LEAVES);
        } else if block::OPAQUE[b as usize] {
            out[SKY] = 0.0;
            break;
        }
    }
    out[SKY] -= leaves as f32 * LEAF_SHADE;

    // Openness: how far, of `WALL_REACH`, the eight compass directions run clear at head height.
    let mut clear = 0;
    for (dx, dz) in [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)] {
        clear += (1..=WALL_REACH)
            .take_while(|&s| !block::OPAQUE[at(cell.x + dx * s, cell.y, cell.z + dz * s) as usize])
            .count();
    }
    out[OPEN] = clear as f32 / (8 * WALL_REACH) as f32;

    // Sea: still water at sea level, in a wide grid. Fresh water and trees: a nearer 3D grid.
    let weight = |dx: i32, dy: i32, dz: i32, reach: i32| {
        (1.0 - ((dx * dx + dy * dy + dz * dz) as f32).sqrt() / reach as f32).max(0.0)
    };
    let mut sea = 0.0;
    for dx in (-SEA_NEAR..=SEA_NEAR).step_by(SEA_STRIDE) {
        for dz in (-SEA_NEAR..=SEA_NEAR).step_by(SEA_STRIDE) {
            if at(cell.x + dx, SEA_LEVEL, cell.z + dz) == block::WATER {
                sea += weight(dx, SEA_LEVEL - cell.y, dz, SEA_NEAR);
            }
        }
    }
    let (mut fresh, mut trees) = (0.0, 0.0);
    for dx in (-NEAR..=NEAR).step_by(NEAR_STRIDE) {
        for dz in (-NEAR..=NEAR).step_by(NEAR_STRIDE) {
            for dy in (BELOW..=ABOVE).step_by(2) {
                let (x, y, z) = (cell.x + dx, cell.y + dy, cell.z + dz);
                let b = at(x, y, z);
                let w = weight(dx, dy, dz, NEAR);
                if block::is_leaves(b) {
                    trees += w;
                } else if b == block::WATER && y > SEA_LEVEL {
                    fresh += w;
                } else if block::flow_level(b).is_some() {
                    fresh += 2.0 * w; // moving water is louder than a pool
                }
            }
        }
    }
    out[SEA] = (sea / SEA_FULL).min(1.0);
    out[FRESH] = (fresh / FRESH_FULL).min(1.0);
    out[TREES] = (trees / TREES_FULL).min(1.0);
    out[DIN] = (factory.din_near(eye, DIN_RADIUS) as f32 / DIN_FULL).min(1.0);
    out[SKY] = out[SKY].max(0.0);
    out
}

#[cfg(test)]
mod tests;
