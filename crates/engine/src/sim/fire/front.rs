//! Localized weed-torch ground scorches (`Front`): clears ground cover right where the flamethrower hits via
//! `world/cover_clear.rs` bits and leaves a brief glowing ember spot that goes out on its own without ever igniting
//! neighbouring ground plants.

use super::{MAX_FRONT, SCORCH_TICKS};
use crate::block::{is_sapling, BlockId, AIR, GRASS, LIQUID, PALE_SOIL, RUSTY_SOIL};
use crate::bytes::{ByteReader, ByteWriter};
use crate::math::{hash3, IVec3};
use crate::sim::Sim;

const CARDINALS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// One short-lived localized ground ember left where the flamethrower scorched turf (never propagates to neighbours).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Front {
    /// Ground block `(x, ground_y, z)` whose surface is smouldering.
    pub pos: IVec3,
    /// Origin column (kept for save-version 82 wire compatibility).
    pub origin: (i32, i32),
    /// Unused for non-spreading scorches (always 0).
    pub heat: u8,
    /// Tick when this ground ember goes out.
    pub due: u64,
}

impl Front {
    pub(super) fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.i32(self.origin.0);
        w.i32(self.origin.1);
        w.u8(self.heat);
        w.u64(self.due);
    }

    pub(super) fn read_state(r: &mut ByteReader) -> Option<Front> {
        Some(Front { pos: r.ivec3()?, origin: (r.i32()?, r.i32()?), heat: r.u8()?, due: r.u64()? })
    }
}

/// Natural turf blocks where the weed torch clears ground cover.
fn is_turf(b: BlockId) -> bool {
    b == GRASS || (RUSTY_SOIL..=PALE_SOIL).contains(&b)
}

impl Sim {
    /// Clears ground plants at `pos`'s column and leaves a short-lived local ember without igniting neighbours.
    pub(crate) fn scorch_ground(&mut self, pos: IVec3) -> bool {
        let Some(gy) = self.turf_surface(pos.x, pos.y, pos.z) else { return false };
        let ground = IVec3::new(pos.x, gy, pos.z);
        if self.water_near_ground(ground) {
            return false;
        }
        let above = ground + IVec3::new(0, 1, 0);
        let burnt_sapling = is_sapling(self.block(above)) && self.burn_sapling(above);
        let cleared = !self.world.is_cover_cleared(pos.x, pos.z);
        self.world.clear_column(pos.x, pos.z);
        if (cleared || burnt_sapling) && self.fire.fronts.len() < MAX_FRONT && !self.fire.has_front(pos.x, pos.z) {
            let seed = self.world.generator().seed();
            let jitter = hash3(seed ^ 0xF007, pos.x, gy, pos.z) as u64 % 18;
            self.fire.insert_front(Front {
                pos: ground,
                origin: (pos.x, pos.z),
                heat: 0,
                due: self.tick + SCORCH_TICKS + jitter,
            });
        }
        cleared || burnt_sapling
    }

    /// Extinguishes one due ground ember cleanly (never spreads to neighbouring plants).
    pub(super) fn step_front(&mut self, f: Front) {
        let above = f.pos + IVec3::new(0, 1, 0);
        if is_sapling(self.block(above)) {
            self.burn_sapling(above);
        }
    }

    /// Finds the y of a turf block near `near_y` whose cell above is air or a sapling.
    fn turf_surface(&mut self, x: i32, near_y: i32, z: i32) -> Option<i32> {
        for y in (near_y - 2..=near_y + 2).rev() {
            let p = IVec3::new(x, y, z);
            if is_turf(self.block(p)) {
                let above = self.block(p + IVec3::new(0, 1, 0));
                if above == AIR || is_sapling(above) {
                    return Some(y);
                }
            }
        }
        None
    }

    /// True if water sits on or horizontally next to surface cell `pos`.
    fn water_near_ground(&mut self, pos: IVec3) -> bool {
        if LIQUID[self.block(pos + IVec3::new(0, 1, 0)) as usize] {
            return true;
        }
        for (dx, dz) in CARDINALS {
            if LIQUID[self.block(IVec3::new(pos.x + dx, pos.y, pos.z + dz)) as usize]
                || LIQUID[self.block(IVec3::new(pos.x + dx, pos.y + 1, pos.z + dz)) as usize]
            {
                return true;
            }
        }
        false
    }
}
