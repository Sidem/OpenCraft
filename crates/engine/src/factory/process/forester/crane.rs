//! A forester's knuckle-boom crane and harvester head, drawn from a [`Pose`] (presentation only). The main boom rises
//! from the slewing column at an angle that flattens as the head reaches further; the stick hangs from its knuckle down
//! to the rotator over the head, telescoping in two stages when the reach is long; hydraulic rams drive both. The head
//! (orange, after a real felling head): a body behind what it grips, two pairs of curved grab arms, two feed rollers,
//! and a chain-saw bar under it that swings through the cut. A tier's crane is longer in proportion to its patch.

use crate::block::tex;
use crate::math::Vec3;

use super::super::super::render::push_framed;
use super::view::{up, Load, Pose, YELLOW};

/// The main boom's and the stick's fixed section as shares of the farthest reach, and how far the stick runs on past
/// its knuckle.
const BOOM: f64 = 0.3;
const STICK: f64 = 0.28;
const TAIL: f64 = 0.4;
/// The head's paint.
const ORANGE: u16 = tex::MAT_EMBER;
const LOGS: [u16; 3] = [tex::LOG_TOP, tex::LOG_SIDE, tex::LOG_TOP];
/// A felled stem's thickness, and a bucked log's.
const STEM: f32 = 0.62;
const LOG: f32 = 0.5;

/// Draws the crane from `pivot` to the head of `pose`; `reach` is the farthest the tier's patch asks of it.
pub(super) fn draw(out: &mut Vec<f32>, pivot: Vec3, pose: &Pose, reach: f64) {
    let frame = Frame::of(pose);
    let wrist = frame.at(0.0, 0.55, -0.68) + up(0.5);
    let flat = Vec3::new(wrist.x - pivot.x, 0.0, wrist.z - pivot.z);
    let out_dir = if flat.length() > 0.05 { flat * (1.0 / flat.length()) } else { pose.toward };
    let lift = 1.2 - 0.85 * (flat.length() / reach).clamp(0.0, 1.0);
    let main = BOOM * reach + 0.4;
    let knuckle = pivot + out_dir * (main * lift.cos()) + up(main * lift.sin());
    let along = (knuckle - pivot) * (1.0 / main);
    let below = unit(up(1.0) - along * along.y);
    // The main boom, its hose and its lift ram from low on the column.
    beam(out, pivot, knuckle, [0.2, 0.24], [YELLOW; 3], false);
    beam(out, pivot + below * -0.15, knuckle + below * -0.15, [0.04, 0.04], [tex::MAT_RUBBER; 3], false);
    ram(out, pivot - up(0.34) + out_dir * 0.1, pivot + (knuckle - pivot) * 0.45 + below * 0.13);
    // The stick: its tail behind the knuckle, the fixed section, then two telescoping stages out to the wrist.
    let span = (wrist - knuckle).length().max(0.1);
    let u = (wrist - knuckle) * (1.0 / span);
    let fixed = STICK * reach;
    let tail = knuckle - u * (TAIL + (fixed - span).max(0.0));
    beam(out, tail, knuckle + u * fixed.min(span), [0.17, 0.2], [tex::MAT_GUNMETAL; 3], false);
    if span > fixed {
        let mid = fixed + (span - fixed) * 0.5;
        beam(out, knuckle + u * (fixed - 0.25), knuckle + u * mid, [0.13, 0.15], [tex::MAT_METAL; 3], false);
        beam(out, knuckle + u * (mid - 0.25), wrist, [0.1, 0.11], [tex::MAT_METAL; 3], false);
    }
    beam(
        out,
        knuckle - Vec3::new(-out_dir.z, 0.0, out_dir.x) * 0.15,
        knuckle + Vec3::new(-out_dir.z, 0.0, out_dir.x) * 0.15,
        [0.26, 0.26],
        [ORANGE; 3],
        true,
    );
    ram(out, pivot + (knuckle - pivot) * 0.62 - below * 0.16, tail + (knuckle - tail) * 0.2 - below * 0.14);
    // The rotator, and its link down to the head.
    beam(out, wrist - up(0.08), wrist + up(0.08), [0.16, 0.16], [tex::MAT_GUNMETAL; 3], true);
    beam(out, wrist, frame.at(0.0, 0.45, -0.7), [0.08, 0.08], [tex::MAT_GUNMETAL; 3], false);
    head(out, &frame, pose);
}

/// The head's own frame: `x` across, `y` along what it grips, `z` from its body towards it.
struct Frame {
    origin: Vec3,
    x: Vec3,
    y: Vec3,
    z: Vec3,
}

impl Frame {
    fn of(pose: &Pose) -> Frame {
        let a = pose.tilt * std::f64::consts::FRAC_PI_2;
        let y = up(a.cos()) + pose.toward * a.sin();
        let z = pose.toward * a.cos() - up(a.sin());
        Frame { origin: pose.head, x: cross(y, z), y, z }
    }

    fn at(&self, x: f64, y: f64, z: f64) -> Vec3 {
        self.origin + self.x * x + self.y * y + self.z * z
    }

    /// A box in the frame, `size` along its x, y and z.
    fn part(&self, out: &mut Vec<f32>, c: [f64; 3], size: [f32; 3], layer: u16, round: bool) {
        push_framed(out, self.at(c[0], c[1], c[2]), [self.x, self.y, self.z], size, [layer; 3], round);
    }
}

/// The felling head: body, valve block, feed rollers, grab arms, saw housing and bar, and what it holds.
fn head(out: &mut Vec<f32>, f: &Frame, pose: &Pose) {
    f.part(out, [0.0, 0.0, -0.68], [0.62, 0.75, 0.36], ORANGE, false);
    f.part(out, [0.0, 0.42, -0.72], [0.3, 0.12, 0.24], tex::MAT_GUNMETAL, false);
    f.part(out, [0.0, -0.46, -0.52], [0.52, 0.16, 0.34], ORANGE, false);
    for side in [-1.0, 1.0] {
        f.part(out, [side * 0.47, 0.0, -0.32], [0.24, 0.46, 0.24], tex::MAT_RUBBER, true);
        // A grab arm: from the body's front corner out round the trunk, curling in to hold it.
        for y in [0.3, -0.22] {
            let s = pose.arms;
            let elbow = [side * (0.68 - 0.1 * s), y, -0.25 + 0.3 * s];
            let tip = [side * (0.85 - 0.6 * s), y, 0.15 + 0.35 * s];
            limb(out, f, [side * 0.34, y, -0.5], elbow);
            limb(out, f, elbow, tip);
        }
    }
    // The saw bar, pivoting at the housing's side: parked across behind the cut, it swings forward through it.
    let swing = pose.saw * 1.65;
    let (pivot, dir) = ([0.4, -0.5, -0.42], f.x * -swing.cos() + f.z * swing.sin());
    let start = f.at(pivot[0], pivot[1], pivot[2]);
    beam(out, start, start + dir * 1.0, [0.03, 0.12], [tex::STEEL; 3], false);
    match pose.load {
        Load::Empty => {}
        Load::Stem(logs) if logs > 0.0 => {
            let (butt, top) = (f.at(0.0, -0.35, 0.0), f.at(0.0, logs - 0.35, 0.0));
            beam(out, butt, top, [STEM, STEM], LOGS, true);
            if logs >= 1.0 {
                f.part(out, [0.0, logs - 0.1, 0.0], [1.7, 1.3, 1.7], tex::LEAVES, false);
            }
        }
        Load::Stem(_) => {}
        Load::Bundle(n) => {
            for k in 0..n.min(6) {
                let (row, layer) = (f64::from(k % 3) - 1.0, f64::from(k / 3));
                let c = f.at(row * f64::from(LOG) * 1.05, 0.15, 0.1 + layer * f64::from(LOG));
                beam(out, c - f.y * 0.5, c + f.y * 0.5, [LOG, LOG], LOGS, true);
            }
        }
        Load::Sapling => {
            f.part(out, [0.0, -0.1, 0.0], [0.08, 0.5, 0.08], tex::LOG_SIDE, false);
            f.part(out, [0.0, 0.15, 0.0], [0.34, 0.3, 0.34], tex::LEAVES, false);
        }
    }
}

/// One segment of a grab arm, between two points of the head's frame.
fn limb(out: &mut Vec<f32>, f: &Frame, a: [f64; 3], b: [f64; 3]) {
    beam(out, f.at(a[0], a[1], a[2]), f.at(b[0], b[1], b[2]), [0.09, 0.1], [ORANGE; 3], false);
}

/// A hydraulic ram from `a` to `b`: a dark barrel over most of it, a bright rod out of it.
fn ram(out: &mut Vec<f32>, a: Vec3, b: Vec3) {
    let mid = a + (b - a) * 0.6;
    beam(out, a, mid, [0.1, 0.1], [tex::MAT_GUNMETAL; 3], true);
    beam(out, mid, b, [0.05, 0.05], [tex::MAT_METAL; 3], true);
}

/// A bar from `a` to `b`, `thick` across and deep (round: a cylinder that wide).
fn beam(out: &mut Vec<f32>, a: Vec3, b: Vec3, thick: [f32; 2], layers: [u16; 3], round: bool) {
    let len = (b - a).length();
    if len < 1e-4 {
        return;
    }
    let y = (b - a) * (1.0 / len);
    let side = if y.y.abs() > 0.99 { Vec3::new(1.0, 0.0, 0.0) } else { unit(cross(y, up(1.0))) };
    let z = cross(side, y);
    push_framed(out, a + (b - a) * 0.5, [side, y, z], [thick[0], len as f32, thick[1]], layers, round);
}

/// `a` × `b`.
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}

fn unit(v: Vec3) -> Vec3 {
    v * (1.0 / v.length().max(1e-9))
}
