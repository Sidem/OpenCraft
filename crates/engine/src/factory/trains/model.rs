//! Where a train is and how it looks: poses along its path, and the boxes of the locomotive and its wagons.
//! Presentation only. A car sits between the poses of its front and its rear (so it follows a bend), at the
//! rails' height. The item models of the locomotive and wagon are these boxes scaled (`item_models/craft.rs`).

use crate::block::tex;
use crate::factory::render::{push_shape, Axis, Form, UPRIGHT};
use crate::factory::smoke;
use crate::math::{IVec3, Vec3};

use super::{Factory, Train, CAR_MM, LOCO_MM};

/// Samples used to turn an arc length along an edge into a curve parameter.
const SAMPLES: usize = 24;
/// The top of the steel rails, from a node's centre (`rail/curve.rs`: rails at -0.38, 0.08 thick).
pub(crate) const RAIL_TOP: f64 = -0.34;

/// One part of a car: x to the right, y above the rails, z backwards from its centre; its size; its texture; a box or a
/// round body (a wheel turns about x, the boiler lies along z).
pub(crate) type CarBox = (f64, f64, f64, [f32; 3], u16, Form);

const B: Form = Form::Box;
/// A wheel or disc facing sideways, and a round body along the car.
const SIDEWAYS: Form = Form::Round(Axis::X, 1.0);
const ALONG: Form = Form::Round(Axis::Z, 1.0);
const WHEEL: [f32; 3] = [0.08, 0.22, 0.22];
const DRIVER: [f32; 3] = [0.08, 0.42, 0.42];
/// The point of a locomotive's chimney mouth, where its smoke starts (x, y, z as in `CarBox`).
const CHIMNEY_MOUTH: [f64; 3] = [0.0, 1.3, -0.85];

/// A steam locomotive, nose at -z: a frame with a red buffer beam, a round green boiler with a dark smokebox door and
/// a headlamp, a chimney, a brass dome, the cab with its roof, three pairs of tall driving wheels joined by coupling
/// rods.
pub(crate) const LOCOMOTIVE_BOXES: [CarBox; 17] = [
    (0.0, 0.3, 0.0, [0.8, 0.14, 2.5], tex::FRAME, B),
    (0.0, 0.32, -1.27, [0.92, 0.16, 0.08], tex::stripe(3), B),
    (0.0, 0.7, -0.35, [0.6, 0.6, 1.5], tex::LOCO_BODY, ALONG),
    (0.0, 0.7, -1.11, [0.48, 0.48, 0.04], tex::SOOT, ALONG),
    (0.0, 1.0, -1.12, [0.14, 0.12, 0.05], tex::LAMP_YELLOW, B),
    (0.0, 1.13, -0.85, [0.16, 0.34, 0.16], tex::SOOT, UPRIGHT),
    (0.0, 1.06, -0.25, [0.22, 0.14, 0.22], tex::COPPER_INGOT, UPRIGHT),
    (0.0, 0.84, 0.8, [0.8, 0.86, 0.8], tex::LOCO_BODY, B),
    (0.0, 1.3, 0.8, [0.92, 0.07, 0.96], tex::FRAME, B),
    (-0.46, 0.21, -0.75, DRIVER, tex::SOOT, SIDEWAYS),
    (0.46, 0.21, -0.75, DRIVER, tex::SOOT, SIDEWAYS),
    (-0.46, 0.21, -0.25, DRIVER, tex::SOOT, SIDEWAYS),
    (0.46, 0.21, -0.25, DRIVER, tex::SOOT, SIDEWAYS),
    (-0.46, 0.21, 0.25, DRIVER, tex::SOOT, SIDEWAYS),
    (0.46, 0.21, 0.25, DRIVER, tex::SOOT, SIDEWAYS),
    (-0.51, 0.21, -0.25, [0.03, 0.05, 1.1], tex::STEEL, B),
    (0.51, 0.21, -0.25, [0.03, 0.05, 1.1], tex::STEEL, B),
];

/// An open wagon: a bed, a steel body with upright ribs, a dark hold seen over a rim of rails, two bogies of two
/// axles each.
pub(crate) const WAGON_BOXES: [CarBox; 21] = [
    (0.0, 0.3, 0.0, [0.8, 0.14, 2.4], tex::FRAME, B),
    (0.0, 0.67, 0.0, [0.76, 0.6, 2.2], tex::STEEL, B),
    (0.0, 0.98, 0.0, [0.66, 0.04, 2.1], tex::SOOT, B),
    (-0.4, 1.0, 0.0, [0.06, 0.06, 2.26], tex::stripe(3), B),
    (0.4, 1.0, 0.0, [0.06, 0.06, 2.26], tex::stripe(3), B),
    (0.0, 0.995, -1.11, [0.84, 0.05, 0.06], tex::stripe(3), B),
    (0.0, 0.995, 1.11, [0.84, 0.05, 0.06], tex::stripe(3), B),
    (-0.39, 0.67, -0.55, [0.04, 0.6, 0.07], tex::FRAME, B),
    (0.39, 0.67, -0.55, [0.04, 0.6, 0.07], tex::FRAME, B),
    (-0.39, 0.67, 0.55, [0.04, 0.6, 0.07], tex::FRAME, B),
    (0.39, 0.67, 0.55, [0.04, 0.6, 0.07], tex::FRAME, B),
    (0.0, 0.17, -0.75, [0.74, 0.1, 0.62], tex::FRAME, B),
    (0.0, 0.17, 0.75, [0.74, 0.1, 0.62], tex::FRAME, B),
    (-0.43, 0.11, -0.97, WHEEL, tex::BELT_TOP, SIDEWAYS),
    (0.43, 0.11, -0.97, WHEEL, tex::BELT_TOP, SIDEWAYS),
    (-0.43, 0.11, -0.53, WHEEL, tex::BELT_TOP, SIDEWAYS),
    (0.43, 0.11, -0.53, WHEEL, tex::BELT_TOP, SIDEWAYS),
    (-0.43, 0.11, 0.53, WHEEL, tex::BELT_TOP, SIDEWAYS),
    (0.43, 0.11, 0.53, WHEEL, tex::BELT_TOP, SIDEWAYS),
    (-0.43, 0.11, 0.97, WHEEL, tex::BELT_TOP, SIDEWAYS),
    (0.43, 0.11, 0.97, WHEEL, tex::BELT_TOP, SIDEWAYS),
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

    /// Draws every car of every train within `range` of the camera at `eye`; a moving locomotive smokes.
    pub(in crate::factory) fn write_train_models(&self, out: &mut Vec<f32>, eye: Vec3, time: f64, range: f64) {
        for (i, t) in self.trains.iter().enumerate() {
            for car in 0..=i64::from(t.cars) {
                let (from, to) =
                    if car == 0 { (0, LOCO_MM) } else { (LOCO_MM + (car - 1) * CAR_MM, LOCO_MM + car * CAR_MM) };
                let (Some(front), Some(rear)) = (self.point_back(t, from), self.point_back(t, to)) else { continue };
                let mid = (front + rear) * 0.5 - eye;
                if mid.length() > range {
                    continue;
                }
                let pose = Pose::new(mid, front - rear);
                pose.draw(out, if car == 0 { &LOCOMOTIVE_BOXES } else { &WAGON_BOXES });
                if car == 0 && t.idle.is_none() {
                    let [x, y, z] = CHIMNEY_MOUTH;
                    smoke::puffs(out, pose.point(x, y, z), time, (i as f64 * 0.37).fract(), 1.0);
                }
            }
        }
    }
}

/// Where a car stands: its camera-relative centre, its turn and slope, and its axes.
struct Pose {
    mid: Vec3,
    yaw: f64,
    pitch: f64,
    right: Vec3,
    up: Vec3,
    fwd: Vec3,
}

impl Pose {
    /// A car centred at camera-relative `mid`, its nose along `d` (rear to front).
    fn new(mid: Vec3, d: Vec3) -> Pose {
        let run = d.x.hypot(d.z).max(1e-9);
        let (yaw, pitch) = (d.x.atan2(-d.z), d.y.atan2(run));
        let (sy, cy, sp, cp) = (yaw.sin(), yaw.cos(), pitch.sin(), pitch.cos());
        let (right, fwd) = (Vec3::new(cy, 0.0, sy), Vec3::new(sy * cp, sp, -cy * cp));
        Pose { mid, yaw, pitch, right, up: Vec3::new(-sy * sp, cp, cy * sp), fwd }
    }

    /// A point of the car (as in `CarBox`), camera-relative.
    fn point(&self, x: f64, y: f64, z: f64) -> Vec3 {
        self.mid + self.right * x + self.up * (RAIL_TOP + y) - self.fwd * z
    }

    fn draw(&self, out: &mut Vec<f32>, boxes: &[CarBox]) {
        for &(x, y, z, size, layer, shape) in boxes {
            push_shape(out, shape, self.point(x, y, z), self.yaw as f32, size, [layer; 3], false);
            // The slope tilts the part about the car's x: a wheel turning about x stays as it is.
            if shape != SIDEWAYS {
                let n = out.len();
                out[n - 4] += self.pitch as f32;
            }
        }
    }
}
