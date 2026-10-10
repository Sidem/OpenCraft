//! Smoke from chimneys, exhausts and the locomotive's stack (presentation only): a few cutout puffs per source,
//! each rising, swelling, drifting downwind and thinning over its life before it starts again at the mouth. A
//! puff is a pure function of the time and its source's seed, so there is nothing to store, save or sync.
//! To make a processor smoke, give it a `Look::Smoke` part at its chimney's mouth (`process/model.rs`); other
//! machines call `puffs` themselves while they burn.

use crate::block::tex;
use crate::math::{hash3, unit, IVec3, Vec3};

use super::render::push_box;

/// Puffs in the air at once per source, how long each lives (seconds), how high it rises and how far the wind
/// takes it (blocks, towards +x).
const PUFFS: usize = 5;
const LIFE: f64 = 2.4;
const RISE: f64 = 2.2;
const WIND: f64 = 0.45;

/// Draws the puffs of a source whose mouth is at camera-relative `at`; `seed` (0..1, `seed`) staggers sources so
/// neighbours don't puff in step, and `scale` sizes them (1 for a chimney, less for a small exhaust).
pub fn puffs(out: &mut Vec<f32>, at: Vec3, time: f64, seed: f64, scale: f32) {
    for k in 0..PUFFS {
        let age = (time / LIFE + seed + k as f64 / PUFFS as f64).fract();
        let sway = 0.1 * (time * 0.9 + k as f64 * 2.1).sin() * age;
        let drift = Vec3::new(WIND * age * age + sway, RISE * age, sway);
        let size = scale * (0.3 + 0.7 * age as f32);
        let layer = if age < 0.55 { tex::SMOKE } else { tex::SMOKE_THIN };
        // Turned square to the camera (at the origin), so its sides show edge-on rather than as slivers.
        let p = at + drift;
        push_box(out, p, p.x.atan2(p.z) as f32, [size; 3], 0.0, [layer; 3], false);
    }
}

/// A seed in 0..1 for the machine at `pos`, to stagger its puffs.
pub fn seed(pos: IVec3) -> f64 {
    unit(hash3(17, pos.x, pos.y, pos.z))
}
