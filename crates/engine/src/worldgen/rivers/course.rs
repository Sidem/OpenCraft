//! Version 8's river courses (R1c): a reach winds from its node to the downstream node instead of running straight,
//! and its water drops where the ground does, in pools of any length.
//!
//! - The course is a line of points every `STEP` blocks of the chord, each pushed sideways by the `meander` noise
//!   sampled there (bends about `BEND` long, up to `SWING` either way). The push fades to nothing at both ends, so
//!   reaches still meet at their nodes, and is gentler where the reach falls steeply (gorges run straighter).
//!   The whole reach also bows to one side (`BOW`). Pushed along one fixed sideways direction, a course never
//!   crosses itself.
//! - Pools: walking the course, the level follows a line between the nodes' levels and dips with the ground (2 under
//!   it), never rising. A drop needs `MIN_POOL` blocks since the last one, so steep ground saves its drops into one
//!   waterfall. The first pool is the upper node's level and the last the lower node's, so reaches agree at joins.
//!
//! Invariants: a pure function of the two nodes and the seed. To tune: the constants below.

use crate::math::{hash2, smoothstep, unit};

use super::super::WorldGen;
use super::{snap, Node};

/// Course points every this many blocks of the chord.
const STEP: f64 = 6.0;
/// The widest sideways push, and the length of a bend (the meander noise's scale).
const SWING: f64 = 30.0;
const BEND: f64 = 64.0;
/// A whole reach also bows to one side by this share of its length (least, most), so none runs node to node straight.
const BOW: (f64, f64) = (0.06, 0.14);
/// The share of the course at each end over which the push fades in.
const FADE: f64 = 0.2;
/// The shortest pool between two drops.
const MIN_POOL: f64 = 7.0;
/// Pieces a bounding box covers when finding the nearest point (a coarse pass first).
const GROUP: usize = 8;

/// A winding river course: its points and the length along it at each.
pub(super) struct Course {
    pts: Vec<(f64, f64)>,
    at: Vec<f64>,
    /// Its bounding box, and each `GROUP` pieces' box.
    pub lo: (f64, f64),
    pub hi: (f64, f64),
    boxes: Vec<((f64, f64), (f64, f64))>,
}

impl Course {
    pub fn len(&self) -> f64 {
        self.at[self.at.len() - 1]
    }

    #[cfg(test)]
    pub fn points(&self) -> &[(f64, f64)] {
        &self.pts
    }

    /// The nearest point of the course to (px, pz) closer than `limit`: (its distance, the length along the course).
    pub fn nearest(&self, px: f64, pz: f64, limit: f64) -> Option<(f64, f64)> {
        let mut best = limit;
        let mut along = None;
        for (g, &(lo, hi)) in self.boxes.iter().enumerate() {
            if px < lo.0 - best || px > hi.0 + best || pz < lo.1 - best || pz > hi.1 + best {
                continue;
            }
            for i in g * GROUP..((g + 1) * GROUP).min(self.pts.len() - 1) {
                let ((ax, az), (bx, bz)) = (self.pts[i], self.pts[i + 1]);
                let (dx, dz) = (bx - ax, bz - az);
                let t = (((px - ax) * dx + (pz - az) * dz) / (dx * dx + dz * dz)).clamp(0.0, 1.0);
                let d = ((px - ax - dx * t).powi(2) + (pz - az - dz * t).powi(2)).sqrt();
                if d < best {
                    best = d;
                    along = Some(self.at[i] + (self.at[i + 1] - self.at[i]) * t);
                }
            }
        }
        along.map(|s| (best, s))
    }
}

/// The bounding box of `pts`.
fn bounds(pts: &[(f64, f64)]) -> ((f64, f64), (f64, f64)) {
    let lo = pts.iter().fold((f64::MAX, f64::MAX), |m, p| (m.0.min(p.0), m.1.min(p.1)));
    let hi = pts.iter().fold((f64::MIN, f64::MIN), |m, p| (m.0.max(p.0), m.1.max(p.1)));
    (lo, hi)
}

impl WorldGen {
    /// The course from node `a` to node `b`.
    pub(super) fn course(&self, a: &Node, b: &Node) -> Course {
        let (ax, az, bx, bz) = (a.x as f64, a.z as f64, b.x as f64, b.z as f64);
        let chord = ((bx - ax).powi(2) + (bz - az).powi(2)).sqrt();
        let (nx, nz) = (-(bz - az) / chord, (bx - ax) / chord);
        let flat = 1.0 - ((a.h - b.h) as f64 / chord * 6.0).clamp(0.0, 0.4);
        let side = unit(hash2(self.seed ^ 0x2160, a.gx, a.gz));
        let bow = chord * (BOW.0 + (BOW.1 - BOW.0) * side) * if side < 0.5 { -1.0 } else { 1.0 };
        let n = (chord / STEP).ceil() as usize;
        let (mut pts, mut at) = (Vec::with_capacity(n + 1), Vec::with_capacity(n + 1));
        for i in 0..=n {
            let u = i as f64 / n as f64;
            let (cx, cz) = (ax + (bx - ax) * u, az + (bz - az) * u);
            let fade = smoothstep(0.0, FADE, u) * smoothstep(0.0, FADE, 1.0 - u);
            let wind = SWING * flat * fade * (2.0 * self.meander.fbm2(cx / BEND, cz / BEND, 3)).clamp(-1.0, 1.0);
            let push = wind + bow * (std::f64::consts::PI * u).sin();
            let p = (cx + nx * push, cz + nz * push);
            let s = pts
                .last()
                .map_or(0.0, |&(qx, qz): &(f64, f64)| at[i - 1] + ((p.0 - qx).powi(2) + (p.1 - qz).powi(2)).sqrt());
            pts.push(p);
            at.push(s);
        }
        let (lo, hi) = bounds(&pts);
        let boxes =
            (0..pts.len() - 1).step_by(GROUP).map(|i| bounds(&pts[i..(i + GROUP + 1).min(pts.len())])).collect();
        Course { pts, at, lo, hi, boxes }
    }

    /// The pools of course `c` from `a` to `b`: where each starts along it (the first at 0) and its level.
    pub(super) fn course_pools(&self, a: &Node, b: &Node, c: &Course) -> (Vec<f64>, Vec<i32>) {
        let len = c.len();
        let (mut ends, mut levels) = (vec![0.0], vec![a.level]);
        let mut last = 0.0;
        for (&(x, z), &s) in c.pts.iter().zip(&c.at) {
            if s < MIN_POOL + last || s > len - MIN_POOL {
                continue;
            }
            let line = (a.level as f64 + (b.level - a.level) as f64 * s / len).round() as i32;
            let want = snap(line.min(self.base_height(x as i32, z as i32) - 2)).max(b.level);
            if want < levels[levels.len() - 1] {
                ends.push(s);
                levels.push(want);
                last = s;
            }
        }
        if levels[levels.len() - 1] > b.level {
            if len - MIN_POOL > last {
                ends.push(len - MIN_POOL);
                levels.push(b.level);
            } else {
                let i = levels.len() - 1;
                levels[i] = b.level;
            }
        }
        (ends, levels)
    }
}
