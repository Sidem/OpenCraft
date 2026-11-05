//! Version 7 landforms (Milestone 10): the terrain noise with larger continents, taller and sharper mountain
//! ranges, stepped mesas in the hot dry regions and a guaranteed dry spawn. Rivers and lakes are carved into this
//! height afterwards (`rivers.rs`); ponds after those (`water.rs`). Version 8 bends the distance from spawn by noise;
//! version 11 flattens what rises near spawn, so no world starts on a mountain (`SPAWN_LOW`).
//!
//! Invariants: a pure function of the seed and the column; versions before 7 never call into this file. The result
//! stays inside `4..=WORLD_HEIGHT - 16` (a soft cap bends the highest peaks under it). To tune: the constants below.

use crate::math::{lerp, smoothstep};

use super::{WorldGen, WORLD_HEIGHT};

/// Heights above this are bent towards it (half as steep), so the tallest peaks stay under the world's roof.
const SOFT_CAP: f64 = 196.0;
/// Mesa terraces: their base height and step.
const MESA_BASE: f64 = 66.0;
const MESA_STEP: f64 = 14.0;
/// Within this many blocks of spawn the ground is lifted to dry land, fading out by the second figure.
const SPAWN_LAND: (f64, f64) = (50.0, 170.0);
const SPAWN_FLOOR: f64 = 66.0;
/// Version 8 bends the distance from spawn within this far (past the mesas' fade even bent 35% short).
const BENT: f64 = 620.0;
/// Version 11's lowland start: within the first figure of spawn, ground above `LOW_BASE` keeps `LOW_KEEP` of its rise,
/// back to all of it by the second (the starter patches, out to 110 blocks, lie in the lowland).
const SPAWN_LOW: (f64, f64) = (120.0, 320.0);
const LOW_BASE: f64 = 68.0;
const LOW_KEEP: f64 = 0.25;

impl WorldGen {
    /// Version 7's terrain height at (x, z), before rivers (version 8 with the spawn ring bent).
    pub(super) fn land_height(&self, x: i32, z: i32) -> i32 {
        let (fx, fz) = (x as f64, z as f64);
        let c = self.continent.fbm2(fx / 1300.0, fz / 1300.0, 4);
        let hills = self.hills.fbm2(fx / 150.0, fz / 150.0, 5);
        let r = self.ridges.fbm2(fx / 420.0, fz / 420.0, 4);
        let ridge = (1.0 - r.abs() * 2.2).max(0.0);
        let mountains = ridge * ridge * smoothstep(-0.05, 0.3, c) * 135.0;
        let mut h = 68.0 + c * 58.0 + hills * 24.0 + mountains;
        // Version 8 bends the distance from spawn, so the lifted ground and the mesas' fade end in no ring.
        // Bent at most 35% either way; beyond `BENT` neither fade can tell, so the noise is skipped there.
        let dist = (fx * fx + fz * fz).sqrt();
        let dist = if self.version >= 8 && dist < BENT {
            dist * (1.0 + 0.35 * self.meander.noise2(fx / 160.0, fz / 160.0).clamp(-1.0, 1.0))
        } else {
            dist
        };
        // Hot dry ground steps up in terraces (the same fields as the desert biome, near spawn faded out).
        let t = self.temperature.fbm2(fx / 1100.0, fz / 1100.0, 2);
        let m = self.moisture.fbm2(fx / 900.0, fz / 900.0, 2);
        let arid = smoothstep(0.05, 0.2, t) * smoothstep(0.05, -0.12, m) * smoothstep(120.0, 400.0, dist);
        if arid > 0.0 && h > MESA_BASE + 4.0 {
            let u = (h - MESA_BASE) / MESA_STEP;
            let step = u.floor() + smoothstep(0.7, 1.0, u - u.floor());
            h = lerp(arid * 0.9, h, MESA_BASE + step * MESA_STEP);
        }
        if h > SOFT_CAP {
            h = SOFT_CAP + (h - SOFT_CAP) * 0.5;
        }
        // Version 11 starts every world in low, gentle country: near spawn whatever rises above the lowland base keeps
        // only `LOW_KEEP` of its height (hills become rolls, ranges become hills), easing back to full by `SPAWN_LOW.1`.
        if self.version >= 11 && h > LOW_BASE {
            let keep = lerp(smoothstep(SPAWN_LOW.0, SPAWN_LOW.1, dist), LOW_KEEP, 1.0);
            h = LOW_BASE + (h - LOW_BASE) * keep;
        }
        h = h.max(SPAWN_FLOOR * smoothstep(SPAWN_LAND.1, SPAWN_LAND.0, dist));
        (h.round() as i32).clamp(4, WORLD_HEIGHT - 16)
    }
}
