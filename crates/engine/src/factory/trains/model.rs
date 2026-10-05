//! Where a train is and how it looks: poses along its path, and the boxes of the locomotive and its wagons.
//! Presentation only. A car sits between the poses of its front and its rear (so it follows a bend), at the
//! rails' height.

use crate::block::tex;
use crate::factory::render::push_box;
use crate::math::{IVec3, Vec3};

use super::{Factory, Train, CAR_MM, LOCO_MM};

/// Samples used to turn an arc length along an edge into a curve parameter.
const SAMPLES: usize = 24;
/// The top of the steel rails, from a node's centre (`rail/curve.rs`: rails at -0.38, 0.08 thick).
const RAIL_TOP: f64 = -0.34;

/// One box of a car: x to the right, y above the rails, z backwards from its centre; its size; its texture.
type CarBox = (f64, f64, f64, [f32; 3], u16);

const WHEEL: [f32; 3] = [0.08, 0.22, 0.22];

const LOCOMOTIVE_BOXES: [CarBox; 13] = [
    (0.0, 0.27, 0.0, [0.8, 0.18, 2.5], tex::FRAME),
    (0.0, 0.64, -0.35, [0.7, 0.55, 1.5], tex::STEEL),
    (0.0, 0.74, 0.85, [0.8, 0.75, 0.8], tex::FRAME),
    (0.0, 1.15, 0.85, [0.92, 0.08, 0.96], tex::stripe(3)),
    (0.0, 0.42, -1.0, [0.5, 0.1, 0.3], tex::stripe(3)),
    (-0.43, 0.11, -0.85, WHEEL, tex::BELT_TOP),
    (0.43, 0.11, -0.85, WHEEL, tex::BELT_TOP),
    (-0.43, 0.11, -0.25, WHEEL, tex::BELT_TOP),
    (0.43, 0.11, -0.25, WHEEL, tex::BELT_TOP),
    (-0.43, 0.11, 0.35, WHEEL, tex::BELT_TOP),
    (0.43, 0.11, 0.35, WHEEL, tex::BELT_TOP),
    (-0.43, 0.11, 0.95, WHEEL, tex::BELT_TOP),
    (0.43, 0.11, 0.95, WHEEL, tex::BELT_TOP),
];

/// An open steel box with a stripe on a flat bed, on four wheels a side.
const WAGON_BOXES: [CarBox; 9] = [
    (0.0, 0.27, 0.0, [0.8, 0.18, 2.4], tex::FRAME),
    (0.0, 0.7, 0.0, [0.76, 0.62, 2.2], tex::STEEL),
    (0.0, 1.03, 0.0, [0.8, 0.06, 2.3], tex::stripe(3)),
    (-0.43, 0.11, -0.7, WHEEL, tex::BELT_TOP),
    (0.43, 0.11, -0.7, WHEEL, tex::BELT_TOP),
    (-0.43, 0.11, 0.7, WHEEL, tex::BELT_TOP),
    (0.43, 0.11, 0.7, WHEEL, tex::BELT_TOP),
    (-0.43, 0.11, 0.0, WHEEL, tex::BELT_TOP),
    (0.43, 0.11, 0.0, WHEEL, tex::BELT_TOP),
];

impl Factory {
    /// The point `back_mm` behind the front of `t`, in world coordinates (a node's cell centre is the line the
    /// track follows); `None` if the track is gone.
    pub(super) fn point_back(&self, t: &Train, back_mm: i64) -> Option<Vec3> {
        let mut left = back_mm;
        let mut at = t.head;
        for &(a, b) in t.path.iter().rev() {
            if left <= at {
                return self.point_on(a, b, at - left);
            }
            left -= at;
            at = self.edge_mm(a, b)?;
        }
        let &(a, b) = t.path.first()?;
        self.point_on(a, b, 0).or_else(|| self.point_on(b, a, 0))
    }

    /// The front of `t`.
    pub(super) fn head_point(&self, t: &Train) -> Option<Vec3> {
        self.point_back(t, 0)
    }

    /// The point `mm` along the track from node `a` towards `b`.
    fn point_on(&self, a: IVec3, b: IVec3, mm: i64) -> Option<Vec3> {
        let curve = self.run(a, b)?;
        let want = mm as f64 / 1000.0;
        let (mut prev, mut so_far) = (curve.at(0.0), 0.0);
        for i in 1..=SAMPLES {
            let (t0, t1) = ((i - 1) as f64 / SAMPLES as f64, i as f64 / SAMPLES as f64);
            let p = curve.at(t1);
            let step = (p - prev).length();
            if so_far + step >= want || i == SAMPLES {
                let f = if step > 1e-9 { ((want - so_far) / step).clamp(0.0, 1.0) } else { 0.0 };
                return Some(curve.at(t0 + (t1 - t0) * f));
            }
            so_far += step;
            prev = p;
        }
        None
    }

    /// Draws every car of every train within `range` of the camera at `eye`.
    pub(in crate::factory) fn write_train_models(&self, out: &mut Vec<f32>, eye: Vec3, range: f64) {
        for t in &self.trains {
            for car in 0..=i64::from(t.cars) {
                let (from, to) =
                    if car == 0 { (0, LOCO_MM) } else { (LOCO_MM + (car - 1) * CAR_MM, LOCO_MM + car * CAR_MM) };
                let (Some(front), Some(rear)) = (self.point_back(t, from), self.point_back(t, to)) else { continue };
                let mid = (front + rear) * 0.5 - eye;
                if mid.length() <= range {
                    draw_car(out, mid, front - rear, if car == 0 { &LOCOMOTIVE_BOXES } else { &WAGON_BOXES });
                }
            }
        }
    }
}

/// A car's boxes centred at camera-relative `mid`, its nose along `d` (rear to front).
fn draw_car(out: &mut Vec<f32>, mid: Vec3, d: Vec3, boxes: &[CarBox]) {
    let run = d.x.hypot(d.z).max(1e-9);
    let (yaw, pitch) = (d.x.atan2(-d.z), d.y.atan2(run));
    let (sy, cy, sp, cp) = (yaw.sin(), yaw.cos(), pitch.sin(), pitch.cos());
    let (right, fwd) = (Vec3::new(cy, 0.0, sy), Vec3::new(sy * cp, sp, -cy * cp));
    let up = Vec3::new(-sy * sp, cp, cy * sp);
    for &(x, y, z, size, layer) in boxes {
        let c = mid + right * x + up * (RAIL_TOP + y) - fwd * z;
        push_box(out, c, yaw as f32, size, 0.0, [layer; 3], false);
        let n = out.len();
        out[n - 4] = pitch as f32;
    }
}
