//! What a forester draws round itself (presentation only): a post-and-rail fence along the edge of the patch it works
//! now, open where the machine stands, so its reach reads at a glance and grows with an upgrade; a red marker post on
//! the first cell of uneven ground that stops it (or stops a bigger patch); the logs bucked off a stem lying by its
//! stump; and the logs in its output stacked on its bunk.

use crate::block::tex;
use crate::math::Vec3;

use super::super::super::render::push_box;
use super::super::super::DIRS;
use super::super::Processor;
use super::view::{toward, up, Pose};
use super::{Forest, Job};

/// Fence posts stand this far apart; the rails run at these heights over the ground.
const POST_EVERY: i32 = 2;
const RAILS: [f64; 2] = [0.38, 0.72];
/// The fence is open this far either side of the machine.
const GATE: f64 = 1.5;
const LOGS: [u16; 3] = [tex::LOG_TOP, tex::LOG_SIDE, tex::LOG_TOP];
/// Logs shown on the bunk at most, and items in the output a shown log stands for.
const BUNK_LOGS: u32 = 6;
const PER_LOG: u32 = 12;

/// Draws `p`'s fence, marker, stump pile and bunk; `rel` is its cell's camera-relative centre.
pub(super) fn draw(out: &mut Vec<f32>, p: &Processor, f: &Forest, rel: Vec3, pose: &Pose, bunk: Vec3) {
    let patch = f.patch(p.pos, p.dir);
    let (d, r) = (DIRS[p.dir as usize % 4].as_vec3(), DIRS[(p.dir as usize + 1) % 4].as_vec3());
    let ground = rel - up(0.5);
    let point = |ahead: f64, side: f64| ground + d * ahead + r * side;
    let (rows, half) = patch.extent();
    let (near, far, left, right) = (0.5, f64::from(rows) + 0.5, -f64::from(half) - 0.5, f64::from(half) + 0.5);
    // Each side as a run from corner to corner, the near side in two runs either side of the gate.
    let runs = [
        (point(near, left), point(far, left)),
        (point(far, left), point(far, right)),
        (point(far, right), point(near, right)),
        (point(near, right), point(near, GATE)),
        (point(near, -GATE), point(near, left)),
    ];
    for (a, b) in runs {
        fence(out, a, b);
    }
    if let Some(cell) = f.ground.blocker {
        let at = rel + (cell - p.pos).as_vec3();
        push_box(out, at + up(0.1), 0.0, [0.1, 1.2, 0.1], 0.0, [tex::LAMP_RED; 3], false);
        push_box(out, at + up(0.55) + r * 0.17, 0.0, [0.02, 0.2, 0.3], 0.0, [tex::LAMP_RED; 3], false);
    }
    // Logs bucked off the stem, lying by the stump until the head gathers them.
    let lying = match f.job {
        Job::Buck | Job::Gather if !matches!(pose.load, super::view::Load::Bundle(_)) => f.cut,
        _ => 0,
    };
    if lying > 0 {
        let stump = rel + (patch.cell(i32::from(f.tree.0), i32::from(f.tree.1), p.pos.y) - p.pos).as_vec3();
        let along = toward(rel, stump, p.dir);
        let across = Vec3::new(-along.z, 0.0, along.x);
        for k in 0..lying.min(6) {
            let (row, layer) = (f64::from(k % 3) - 1.0, f64::from(k / 3));
            let c = stump - up(0.5) + across * (1.1 + row * 0.52) + along * 0.5 + up(0.26 + layer * 0.48);
            log(out, c, along, 1.0, 0.5);
        }
    }
    let stacked = p.out.total().div_ceil(PER_LOG).min(BUNK_LOGS);
    let yaw_z = Vec3::new(0.0, 0.0, 1.0);
    let (along, across) = (rotate(yaw_z, p.dir), rotate(Vec3::new(1.0, 0.0, 0.0), p.dir));
    for k in 0..stacked {
        let (row, layer) = (f64::from(k % 3) - 1.0, f64::from(k / 3));
        log(out, bunk + across * (row * 0.12) + up(0.08 + layer * 0.11), along, 0.4, 0.12);
    }
}

/// A run of fence from `a` to `b` on the ground: posts every `POST_EVERY` blocks (one at each end) and two rails.
fn fence(out: &mut Vec<f32>, a: Vec3, b: Vec3) {
    let len = (b - a).length();
    if len < 0.1 {
        return;
    }
    let u = (b - a) * (1.0 / len);
    let yaw = (-u.x).atan2(u.z) as f32;
    let posts = (len / f64::from(POST_EVERY)).round().max(1.0) as i32;
    for k in 0..=posts {
        let at = a + (b - a) * (f64::from(k) / f64::from(posts));
        push_box(out, at + up(0.45), yaw, [0.13, 0.9, 0.13], 0.0, LOGS, false);
    }
    for h in RAILS {
        push_box(out, a + (b - a) * 0.5 + up(h), yaw, [0.05, 0.1, len as f32], 0.0, [tex::PLANKS; 3], false);
    }
}

/// A log `len` long and `thick` across, lying along `along` with its middle at `c`.
fn log(out: &mut Vec<f32>, c: Vec3, along: Vec3, len: f32, thick: f32) {
    let yaw = (-along.x).atan2(along.z) as f32;
    push_box(out, c, yaw, [thick, thick, len], 0.0, LOGS, false);
    super::super::super::render::make_round(out, super::super::super::render::Axis::Z, 1.0);
}

/// `v` turned as a machine facing `dir` turns its model (`model::local`).
fn rotate(v: Vec3, dir: u8) -> Vec3 {
    let yaw = f64::from(dir) * std::f64::consts::FRAC_PI_2;
    let (s, c) = (yaw.sin(), yaw.cos());
    Vec3::new(c * v.x - s * v.z, v.y, s * v.x + c * v.z)
}
