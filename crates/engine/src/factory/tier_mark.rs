//! The Mk mark every tiered machine wears (`tiers.rs` families): one square stud per Mk in the tier's paint
//! (`tex::paint`), so a tier reads by count as well as by colour. Studs are geometry, not texture: a stripe's pips
//! were squeezed into a thin band (0.6 px at 16 blocks); a stud is `STUD` square wherever it sits (5 px at 16
//! blocks on a 1080p screen, at the default field of view). Presentation only.
//!
//! Invariants: a row of `MAX_STUDS` fits `ROW_MAX` across, which every host face is at least (tested per kind);
//! marks are drawn only within `DETAIL` blocks of the camera, where they are still a few pixels; farther out the
//! tier's coloured band or stripe is the cue.
//!
//! To mark a new tiered kind: call `ring` around its band or housing (or `collars` up a post) from its model, with
//! its centre relative to the camera; a kind with a frame of its own places `offsets` itself (`miner_model.rs`).

use crate::block::tex;
use crate::math::Vec3;

use super::render::push_box;

/// A stud's side, its gap to the next and how far it stands proud of its face, in blocks.
pub const STUD: f32 = 0.11;
pub const GAP: f32 = 0.04;
pub const DEPTH: f32 = 0.03;
/// The most studs a row holds (Mk5) and how wide that row is.
pub const MAX_STUDS: u8 = 5;
#[cfg(test)]
pub const ROW_MAX: f32 = MAX_STUDS as f32 * STUD + (MAX_STUDS - 1) as f32 * GAP;
/// Marks are drawn within this many blocks of the camera (3 px a stud there; at 40 blocks they were 2 px and cost a
/// quarter more in `bench_machine_marks`).
pub const DETAIL: f64 = 24.0;
/// A straight belt run shows its Mk on one belt in this many (`belt.rs`): enough to read a run, few enough not to
/// line every belt with studs.
pub const BELT_EVERY: i32 = 6;

/// Whether a point `rel` from the camera is near enough for marks.
pub fn near(rel: Vec3) -> bool {
    rel.x * rel.x + rel.y * rel.y + rel.z * rel.z <= DETAIL * DETAIL
}

/// The centres along a row of `tier + 1` studs, centred on 0.
pub fn offsets(tier: u8) -> impl Iterator<Item = f32> {
    let n = tier.min(MAX_STUDS - 1) + 1;
    let first = -((n - 1) as f32) * (STUD + GAP) / 2.0;
    (0..n).map(move |i| first + i as f32 * (STUD + GAP))
}

/// Studs on the sides of a box centred at `at` (relative to the camera) whose half sizes across are `half` (x, z, in
/// its own frame), turned by `yaw`. Only the sides that face the camera (at most two): the others are out of sight.
pub fn ring(out: &mut Vec<f32>, at: Vec3, yaw: f32, half: [f32; 2], tier: u8) {
    if !near(at) {
        return;
    }
    let layer = [tex::paint(tier); 3];
    let (s, c) = (yaw.sin() as f64, yaw.cos() as f64);
    let turn = |x: f32, z: f32| Vec3::new(c * x as f64 - s * z as f64, 0.0, s * x as f64 + c * z as f64);
    let [hx, hz] = [half[0] + DEPTH / 2.0, half[1] + DEPTH / 2.0];
    for side in [-1.0, 1.0] {
        for (normal, size) in [(turn(0.0, side), [STUD, STUD, DEPTH]), (turn(side, 0.0), [DEPTH, STUD, STUD])] {
            // The side faces the camera when the camera lies beyond it: its normal points back at the eye.
            let face = at + normal * if size[2] == DEPTH { hz } else { hx } as f64;
            if normal.x * face.x + normal.z * face.z >= 0.0 {
                continue;
            }
            for o in offsets(tier) {
                let along = if size[2] == DEPTH { turn(o, 0.0) } else { turn(0.0, o) };
                push_box(out, face + along, yaw, size, 0.0, layer, false);
            }
        }
    }
}

/// Collars up a post `width` across: `tier + 1` of them, the lowest centred at `at` (relative to the camera).
pub fn collars(out: &mut Vec<f32>, at: Vec3, width: f32, tier: u8) {
    if !near(at) {
        return;
    }
    let size = [width + 2.0 * DEPTH, STUD * 0.7, width + 2.0 * DEPTH];
    for i in 0..=tier.min(MAX_STUDS - 1) {
        let up = Vec3::new(0.0, i as f64 * (STUD + GAP) as f64, 0.0);
        push_box(out, at + up, 0.0, size, 0.0, [tex::paint(tier); 3], false);
    }
}

#[cfg(test)]
mod tests;
