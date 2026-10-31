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

/// One part of a car: x to the right, y above the rails, z backwards from its centre; its size; its texture; a box, a
/// box leaning back (the windscreen) or a round body (a wheel turns about x, the fuel tank lies along z).
pub(crate) type CarBox = (f64, f64, f64, [f32; 3], u16, Form);

const B: Form = Form::Box;
/// A wheel or disc facing sideways, and a round body along the car.
const SIDEWAYS: Form = Form::Round(Axis::X, 1.0);
const ALONG: Form = Form::Round(Axis::Z, 1.0);
const WHEEL: [f32; 3] = [0.08, 0.24, 0.24];
/// A wheel's radius (blocks): it turns by the distance driven over this (`Pose::draw`).
const WHEEL_R: f64 = 0.12;
/// The point of a locomotive's exhaust stack, where its smoke starts (x, y, z as in `CarBox`), and the puffs' size.
const CHIMNEY_MOUTH: [f64; 3] = [0.0, 1.32, -0.1];
const PUFF: f32 = 0.35;
/// A wagon's heap of goods: where the hold's floor is, and how high the heap stands empty and full (CarBox y).
const HOLD: f64 = 0.992;
const HEAP_MIN: f64 = 0.04;
const HEAP_HIGH: f64 = 0.26;
/// How far back the cab's windscreen leans (radians).
const RAKE: f32 = 0.346;

const fn wheel(x: f64, z: f64) -> CarBox {
    (x, 0.12, z, WHEEL, tex::WHEEL, SIDEWAYS)
}

/// A modern freight locomotive, nose at -z, in the blue and amber livery: a short nose with headlamps under a raked
/// windscreen, the cab with side windows, a long hood with a radiator, two fans and an exhaust stack, handrails, red
/// tail lamps, a fuel tank slung between two three-axle bogies.
pub(crate) const LOCOMOTIVE_BOXES: [CarBox; 37] = [
    (0.0, 0.34, 0.0, [0.86, 0.1, 2.36], tex::FRAME, B),
    (0.0, 0.3, -1.2, [0.88, 0.14, 0.06], tex::MAT_GUNMETAL, B),
    (0.0, 0.3, 1.2, [0.88, 0.14, 0.06], tex::MAT_GUNMETAL, B),
    (0.0, 0.2, 0.0, [0.6, 0.17, 0.62], tex::MAT_GUNMETAL, ALONG),
    (0.0, 0.16, -0.72, [0.74, 0.12, 0.76], tex::FRAME, B),
    (0.0, 0.16, 0.72, [0.74, 0.12, 0.76], tex::FRAME, B),
    wheel(-0.43, -0.98),
    wheel(0.43, -0.98),
    wheel(-0.43, -0.72),
    wheel(0.43, -0.72),
    wheel(-0.43, -0.46),
    wheel(0.43, -0.46),
    wheel(-0.43, 0.46),
    wheel(0.43, 0.46),
    wheel(-0.43, 0.72),
    wheel(0.43, 0.72),
    wheel(-0.43, 0.98),
    wheel(0.43, 0.98),
    // The long hood, the cab, the nose and the raked front with its windscreen.
    (0.0, 0.76, 0.345, [0.74, 0.74, 1.59], tex::LOCO_BODY, B),
    (0.0, 0.845, -0.635, [0.84, 0.93, 0.43], tex::LOCO_BODY, B),
    (0.0, 0.5925, -1.015, [0.8, 0.415, 0.37], tex::LOCO_BODY, B),
    (0.0, 0.999, -0.769, [0.82, 0.531, 0.3], tex::LOCO_BODY, Form::Lean(RAKE)),
    (0.0, 1.06, -0.938, [0.66, 0.42, 0.04], tex::CAB_GLASS, Form::Lean(RAKE)),
    (-0.425, 1.1, -0.66, [0.02, 0.22, 0.3], tex::CAB_GLASS, B),
    (0.425, 1.1, -0.66, [0.02, 0.22, 0.3], tex::CAB_GLASS, B),
    (0.0, 1.325, -0.62, [0.86, 0.04, 0.5], tex::MAT_GUNMETAL, B),
    (0.15, 1.365, -0.55, [0.06, 0.04, 0.12], tex::STEEL, B),
    (-0.25, 0.68, -1.205, [0.12, 0.08, 0.02], tex::LAMP_YELLOW, B),
    (0.25, 0.68, -1.205, [0.12, 0.08, 0.02], tex::LAMP_YELLOW, B),
    // Roof gear: the radiator and its fans, the exhaust stack.
    (0.0, 1.155, 0.78, [0.62, 0.05, 0.56], tex::MAT_METAL, B),
    (0.0, 1.19, 0.64, [0.24, 0.03, 0.24], tex::FRAME, UPRIGHT),
    (0.0, 1.19, 0.92, [0.24, 0.03, 0.24], tex::FRAME, UPRIGHT),
    (0.0, 1.22, -0.1, [0.1, 0.18, 0.1], tex::SOOT, UPRIGHT),
    (-0.3, 0.6, 1.145, [0.08, 0.06, 0.02], tex::LAMP_RED, B),
    (0.3, 0.6, 1.145, [0.08, 0.06, 0.02], tex::LAMP_RED, B),
    (-0.41, 0.62, 0.35, [0.02, 0.02, 1.5], tex::STEEL, B),
    (0.41, 0.62, 0.35, [0.02, 0.02, 1.5], tex::STEEL, B),
];

/// An open wagon in the locomotive's livery: a frame, the body with a dark hold under a gunmetal rim, two bogies of two
/// axles each.
pub(crate) const WAGON_BOXES: [CarBox; 17] = [
    (0.0, 0.3, 0.0, [0.8, 0.1, 2.36], tex::FRAME, B),
    (0.0, 0.67, 0.0, [0.82, 0.62, 2.24], tex::LOCO_BODY, B),
    (0.0, 0.985, 0.0, [0.72, 0.01, 2.14], tex::SOOT, B),
    (-0.4, 0.99, 0.0, [0.06, 0.04, 2.26], tex::MAT_GUNMETAL, B),
    (0.4, 0.99, 0.0, [0.06, 0.04, 2.26], tex::MAT_GUNMETAL, B),
    (0.0, 0.99, -1.11, [0.84, 0.05, 0.06], tex::MAT_GUNMETAL, B),
    (0.0, 0.99, 1.11, [0.84, 0.05, 0.06], tex::MAT_GUNMETAL, B),
    (0.0, 0.16, -0.75, [0.72, 0.12, 0.66], tex::FRAME, B),
    (0.0, 0.16, 0.75, [0.72, 0.12, 0.66], tex::FRAME, B),
    wheel(-0.43, -0.96),
    wheel(0.43, -0.96),
    wheel(-0.43, -0.54),
    wheel(0.43, -0.54),
    wheel(-0.43, 0.54),
    wheel(0.43, 0.54),
    wheel(-0.43, 0.96),
    wheel(0.43, 0.96),
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

    /// Where car `car` of `t` stands (0 the locomotive, then the wagons), relative to `eye`. Not inlined: drawing,
    /// aiming and the outline each call it (one copy in the wasm).
    #[inline(never)]
    pub(super) fn car_pose(&self, t: &Train, car: u8, eye: Vec3) -> Option<Pose> {
        let car = i64::from(car);
        let (from, to) = if car == 0 { (0, LOCO_MM) } else { (LOCO_MM + (car - 1) * CAR_MM, LOCO_MM + car * CAR_MM) };
        let (front, rear) = (self.point_back(t, from)?, self.point_back(t, to)?);
        Some(Pose::new((front + rear) * 0.5 - eye, front - rear))
    }

    /// Draws every car of every train within `range` of the camera at `eye`; a moving locomotive smokes, and an open
    /// wagon shows a heap of what it carries most of, as high as it is full.
    pub(in crate::factory) fn write_train_models(&self, out: &mut Vec<f32>, eye: Vec3, time: f64, range: f64) {
        for (i, t) in self.trains.iter().enumerate() {
            for car in 0..=t.cars {
                let Some(pose) = self.car_pose(t, car, eye).filter(|p| p.mid.length() <= range) else { continue };
                let spin = (-f64::from(t.odo) / 1000.0 / WHEEL_R).rem_euclid(std::f64::consts::TAU) as f32;
                pose.draw(out, if car == 0 { &LOCOMOTIVE_BOXES } else { &WAGON_BOXES }, spin);
                if let Some((layer, fill)) = Factory::heap(t, car) {
                    let (h, crown) = (HEAP_MIN + HEAP_HIGH * fill, (HEAP_MIN + HEAP_HIGH * fill) * 0.5);
                    pose.draw(out, &[(0.0, HOLD + h * 0.5, 0.0, [0.7, h as f32, 2.1], layer, B)], 0.0);
                    pose.draw(out, &[(0.0, HOLD + h + crown * 0.5, 0.0, [0.42, crown as f32, 1.5], layer, B)], 0.0);
                }
                if car == 0 && t.idle.is_none() {
                    let [x, y, z] = CHIMNEY_MOUTH;
                    smoke::puffs(out, pose.point(x, y, z), time, (i as f64 * 0.37).fract(), PUFF);
                }
            }
        }
    }
}

/// Where a car stands: its centre (camera-relative, or in the world for an eye at the origin), its turn and slope, and
/// its axes.
pub(super) struct Pose {
    pub(super) mid: Vec3,
    yaw: f64,
    pitch: f64,
    pub(super) right: Vec3,
    pub(super) up: Vec3,
    pub(super) fwd: Vec3,
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

    /// Draws `boxes` turned with the car, its wheels turned `spin` about their axles (a round body's roll).
    fn draw(&self, out: &mut Vec<f32>, boxes: &[CarBox], spin: f32) {
        for &(x, y, z, size, layer, shape) in boxes {
            push_shape(out, shape, self.point(x, y, z), self.yaw as f32, size, [layer; 3], false);
            // The slope tilts the part about the car's x: a wheel turning about x turns instead.
            let n = out.len();
            if shape == SIDEWAYS {
                out[n - 3] = spin;
            } else {
                out[n - 4] += self.pitch as f32;
            }
        }
    }
}
