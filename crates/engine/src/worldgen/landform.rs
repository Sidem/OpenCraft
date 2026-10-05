//! Version 7 landforms (Milestone 10): the terrain noise with larger continents, taller and sharper mountain
//! ranges, stepped mesas in the hot dry regions and a guaranteed dry spawn. Rivers and lakes are carved into this
//! height afterwards (`rivers.rs`); ponds after those (`water.rs`).
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

impl WorldGen {
    /// Version 7's terrain height at (x, z), before rivers.
    pub(super) fn land_height(&self, x: i32, z: i32) -> i32 {
        let (fx, fz) = (x as f64, z as f64);
        let c = self.continent.fbm2(fx / 1300.0, fz / 1300.0, 4);
        let hills = self.hills.fbm2(fx / 150.0, fz / 150.0, 5);
        let r = self.ridges.fbm2(fx / 420.0, fz / 420.0, 4);
        let ridge = (1.0 - r.abs() * 2.2).max(0.0);
        let mountains = ridge * ridge * smoothstep(-0.05, 0.3, c) * 135.0;
        let mut h = 68.0 + c * 58.0 + hills * 24.0 + mountains;
        let dist = (fx * fx + fz * fz).sqrt();
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
        h = h.max(SPAWN_FLOOR * smoothstep(SPAWN_LAND.1, SPAWN_LAND.0, dist));
        (h.round() as i32).clamp(4, WORLD_HEIGHT - 16)
    }
}
