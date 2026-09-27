//! The quarry's model (presentation only): a housing with a status lamp, corner posts around its box,
//! rails along the sides, a gantry beam that travels from row to row, and a trolley whose drill
//! reaches down to the block being dug, spinning while it digs. The head glides from the last dug
//! block to the next over the first `TRAVEL` of each block's dig time.

use crate::block::tex;
use crate::math::Vec3;

use super::super::render::push_box;
use super::super::{ticks, DIRS};
use super::{Quarry, QuarryStatus, DIG_SECONDS};
use crate::factory::power::FULL_SPEED;

/// How high the gantry runs above the quarry's floor.
const GANTRY: f64 = 2.6;
/// Share of each block's dig time the head spends travelling to it.
const TRAVEL: f64 = 0.3;

pub(super) fn draw(q: &Quarry, out: &mut Vec<f32>, rel: Vec3, time: f64) {
    let centre = q.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
    let at = |p: Vec3| rel + (p - centre);
    let frame = [tex::FRAME; 3];

    let housing = [tex::STEEL, tex::MINER_MK2_SIDE, tex::FRAME];
    push_box(out, rel + Vec3::new(0.0, -0.2, 0.0), 0.0, [0.9, 0.6, 0.9], 0.0, housing, false);
    push_box(out, rel + Vec3::new(0.0, 0.2, 0.0), 0.0, [0.55, 0.22, 0.55], 0.0, frame, false);
    push_box(out, rel + Vec3::new(0.3, 0.36, 0.3), 0.0, [0.14, 0.1, 0.14], 0.0, [lamp(q.status); 3], false);

    let dig = q.dig_box();
    let (lo, hi) = dig.bounds();
    let (x0, x1, z0, z1) = (lo.x as f64, hi.x as f64 + 1.0, lo.z as f64, hi.z as f64 + 1.0);
    let (floor, beam) = (dig.top as f64, dig.top as f64 + GANTRY);
    for (x, z) in [(x0, z0), (x1, z0), (x0, z1), (x1, z1)] {
        push_box(out, at(Vec3::new(x, floor + GANTRY / 2.0, z)), 0.0, [0.18, GANTRY as f32, 0.18], 0.0, frame, true);
    }
    // Rails run the way the box lies; the beam spans across it.
    let along_z = DIRS[q.facing as usize].z != 0;
    let (mx, mz) = ((x0 + x1) / 2.0, (z0 + z1) / 2.0);
    let (lx, lz) = ((x1 - x0) as f32, (z1 - z0) as f32);
    for s in [0.0, 1.0] {
        let (c, size) = if along_z {
            (Vec3::new(x0 + (x1 - x0) * s, beam, mz), [0.16, 0.16, lz])
        } else {
            (Vec3::new(mx, beam, z0 + (z1 - z0) * s), [lx, 0.16, 0.16])
        };
        push_box(out, at(c), 0.0, size, 0.0, frame, true);
    }

    let t = (q.progress as f64 / (ticks(DIG_SECONDS) as f64 * FULL_SPEED as f64 * TRAVEL)).min(1.0);
    let (a, b) = (q.from.as_vec3(), q.head.as_vec3());
    let h = a + (b - a) * t + Vec3::new(0.5, 0.0, 0.5);
    let (c, size) = if along_z {
        (Vec3::new(mx, beam, h.z), [lx, 0.22, 0.26])
    } else {
        (Vec3::new(h.x, beam, mz), [0.26, 0.22, lz])
    };
    push_box(out, at(c), 0.0, size, 0.0, [tex::STEEL; 3], true);
    push_box(out, at(Vec3::new(h.x, beam + 0.05, h.z)), 0.0, [0.55, 0.35, 0.55], 0.0, housing, false);

    let working = q.status == QuarryStatus::Digging && q.progress > 0;
    let tip = h.y + 1.15;
    let len = (beam - tip).max(0.1);
    let spin = if working { (time * 9.0) as f32 } else { 0.0 };
    push_box(out, at(Vec3::new(h.x, tip + len / 2.0, h.z)), spin, [0.14, len as f32, 0.14], 0.0, frame, false);
    let dip = if working { 0.06 * (time * 14.0).sin() } else { 0.0 };
    let bit = [tex::DRILL; 3];
    push_box(out, at(Vec3::new(h.x, h.y + 1.05 + dip, h.z)), spin, [0.42, 0.22, 0.42], 0.0, bit, true);
    push_box(out, at(Vec3::new(h.x, h.y + 0.9 + dip, h.z)), spin, [0.2, 0.16, 0.2], 0.0, bit, true);
}

/// The status lamp: green digging, yellow output full, red no power, blue flooded.
fn lamp(status: QuarryStatus) -> u16 {
    match status {
        QuarryStatus::Digging => tex::LAMP_GREEN,
        QuarryStatus::OutputFull => tex::LAMP_YELLOW,
        QuarryStatus::NoPower => tex::LAMP_RED,
        QuarryStatus::Flooded => tex::LAMP_BLUE,
        QuarryStatus::Paused | QuarryStatus::Done => tex::FRAME,
    }
}
