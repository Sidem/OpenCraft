//! The pumpjack's model (presentation only), after a classic beam pumping unit: a concrete pad and a steel skid; at
//! the back an electric motor driving a gearbox through a belt guard, the gearbox's crank arms carrying counterweights
//! and the wrist pins; a samson post (an A-frame) carrying the walking beam on a saddle bearing; at the beam's front a
//! curved horsehead, its bridle hanging straight down to the carrier bar and the polished rod that runs into the
//! wellhead's stuffing box; at its back the equalizer, hung from the crank pins by two pitman arms.
//!
//! The motion is the real linkage, solved each frame ([`pose`]): the crank turns at `Pump::crank` (blended into the
//! tick, so it glides), its pins swing the pitman arms, and the beam is wherever the pitman arm's length puts its tail
//! on the circle about the saddle bearing. The horsehead's face is an arc about that bearing, so the bridle always
//! hangs plumb over the well and the rod rises by the arc it unwinds. Model units are blocks, origin the footprint's
//! centre, front (the well) towards +z; a compact pumpjack (`pump.rs`) is the same drawn at `COMPACT` scale.
//!
//! To change the look: the constants (the linkage's points) and the rows in `draw`. `tests.rs` holds the linkage to
//! its lengths and keeps every part inside the footprint.

use std::f64::consts::TAU;

use crate::block::tex;
use crate::math::Vec3;
use crate::TICK;

use super::super::super::render::{push_framed, push_shape, Axis, Form, Framed, UPRIGHT};
use super::super::model::lamp;
use super::super::{Extra, Processor};

/// The saddle bearing the beam rocks on, (y, z).
pub(super) const PIVOT: [f64; 2] = [0.72, 0.2];
/// The horsehead's face: its radius about the bearing, and the angles (from the beam, upwards positive) it spans.
pub(super) const HEAD: f64 = 0.95;
const HEAD_ARC: [f64; 2] = [-0.52, 0.42];
/// From the bearing back to the equalizer.
pub(super) const TAIL: f64 = 0.75;
/// The crank shaft, (y, z), and the wrist pins' radius on it.
pub(super) const CRANK: [f64; 2] = [-0.62, -0.55];
pub(super) const CRANK_R: f64 = 0.26;
/// The carrier bar hangs this far below the bearing with the beam level; the polished rod below it.
const BRIDLE: f64 = 1.05;
const ROD: f64 = 0.78;
/// A compact pumpjack's scale: its 3 blocks deep and high in one.
const COMPACT: f64 = 1.0 / 3.0;
/// Paint: a green frame and beam, grey iron, a yellow guard (the block's own two layers, flat paint: the placed jack
/// never shows its block faces).
const GREEN: u16 = tex::PUMPJACK_SIDE;
const YELLOW: u16 = tex::PUMPJACK_TOP;
const IRON: u16 = tex::MAT_GUNMETAL;
const STEEL: u16 = tex::MAT_METAL;

/// Where the moving parts are at one angle of the crank: (y, z) points in the model's side view.
#[derive(Clone, Copy, Debug)]
pub(super) struct Pose {
    /// The beam's tilt, radians, the horsehead up positive.
    pub beam: f64,
    /// A wrist pin, and the equalizer at the beam's tail.
    pub pin: [f64; 2],
    pub tail: [f64; 2],
}

/// The linkage at crank angle `crank` (radians).
pub(super) fn pose(crank: f64) -> Pose {
    let pin = [CRANK[0] + CRANK_R * crank.sin(), CRANK[1] + CRANK_R * crank.cos()];
    // The tail is where the circle about the bearing (radius TAIL) meets the circle about the pin (the pitman arm):
    // of the two crossings, the one behind.
    let (dy, dz) = (pin[0] - PIVOT[0], pin[1] - PIVOT[1]);
    let d = (dy * dy + dz * dz).sqrt();
    let a = (TAIL * TAIL - pitman() * pitman() + d * d) / (2.0 * d);
    let h = (TAIL * TAIL - a * a).max(0.0).sqrt();
    let base = [PIVOT[0] + dy * a / d, PIVOT[1] + dz * a / d];
    let cross = [base[0] + dz * h / d, base[1] - dy * h / d];
    let other = [base[0] - dz * h / d, base[1] + dy * h / d];
    let tail = if cross[1] < other[1] { cross } else { other };
    Pose { beam: (PIVOT[0] - tail[0]).atan2(PIVOT[1] - tail[1]), pin, tail }
}

/// The pitman arm's length: the beam is level with the pins at the crank's front and back.
pub(super) fn pitman() -> f64 {
    (CRANK_R * CRANK_R + (PIVOT[0] - CRANK[0]).powi(2)).sqrt()
}

/// Draws `p` about `centre` (its footprint's centre, camera-relative) turned by `yaw`, at render `time`.
pub fn draw(p: &Processor, out: &mut Vec<f32>, centre: Vec3, yaw: f32, time: f64) {
    let Extra::Pump(pump) = &p.extra else { return };
    let turn = pump.crank + (time / TICK).fract() * pump.spin;
    let scale = if pump.compact { COMPACT } else { 1.0 };
    let m = Model { out, centre, yaw: f64::from(yaw), scale };
    m.draw(pose(turn * TAU), lamp(p.status));
}

/// Pushes the parts of one pumpjack: positions are model `[x, y, z]`.
struct Model<'a> {
    out: &'a mut Vec<f32>,
    centre: Vec3,
    yaw: f64,
    scale: f64,
}

impl Model<'_> {
    fn draw(mut self, pose: Pose, lamp: u16) {
        self.base();
        self.drive(lamp);
        self.samson_post();
        self.wellhead();
        self.crank(pose);
        self.beam(pose);
    }

    /// The pad, the skid's rails and cross members, and the plinths under the gearbox and motor.
    fn base(&mut self) {
        self.fixed([0.0, -1.45, 0.0], [1.8, 0.1, 2.86], [tex::CONCRETE; 3], Form::Box);
        for x in [-0.42, 0.42] {
            self.fixed([x, -1.33, -0.225], [0.12, 0.14, 2.35], [IRON; 3], Form::Box);
        }
        for z in [-1.3, 0.85] {
            self.fixed([0.0, -1.34, z], [0.9, 0.1, 0.1], [IRON; 3], Form::Box);
        }
        self.fixed([0.0, -1.22, -0.6], [0.9, 0.08, 0.7], [IRON; 3], Form::Box);
        self.fixed([0.0, -1.22, -1.28], [0.9, 0.08, 0.26], [IRON; 3], Form::Box);
    }

    /// The gearbox with its round top and crank shaft, the motor behind it, the belt guard between them, and the
    /// motor's starter box with the status lamp.
    fn drive(&mut self, lamp: u16) {
        self.fixed([0.0, -0.95, -0.55], [0.42, 0.46, 0.56], [GREEN; 3], Form::Box);
        self.fixed([0.0, -0.72, -0.55], [0.4, 0.56, 0.56], [GREEN; 3], Form::Round(Axis::X, 1.0));
        self.fixed([0.0, CRANK[0], CRANK[1]], [0.66, 0.1, 0.1], [STEEL; 3], Form::Round(Axis::X, 1.0));
        self.fixed([0.0, -1.03, -1.28], [0.28, 0.28, 0.28], [STEEL; 3], Form::Round(Axis::X, 1.0));
        self.fixed([-0.18, -0.95, -1.075], [0.06, 0.26, 0.47], [YELLOW; 3], Form::Box);
        self.fixed([0.0, -0.83, -1.28], [0.14, 0.12, 0.12], [IRON; 3], Form::Box);
        self.fixed([0.0, -0.73, -1.28], [0.08, 0.08, 0.08], [lamp; 3], Form::Box);
    }

    /// Four legs from the rails to the saddle, braced, and the saddle bearing on top.
    fn samson_post(&mut self) {
        for x in [-1.0, 1.0] {
            self.bar([0.42 * x, -1.26, 0.85], [0.13 * x, 0.58, 0.24], [0.09, 0.09], GREEN);
            self.bar([0.42 * x, -1.26, 0.06], [0.13 * x, 0.58, 0.16], [0.09, 0.09], GREEN);
            self.bar([0.3 * x, -0.5, 0.6], [0.3 * x, -0.5, 0.1], [0.06, 0.06], GREEN);
        }
        self.bar([-0.27, -0.7, 0.67], [0.27, -0.7, 0.67], [0.06, 0.06], GREEN);
        self.fixed([0.0, 0.63, PIVOT[1]], [0.34, 0.1, 0.2], [IRON; 3], Form::Box);
        self.fixed([0.0, PIVOT[0], PIVOT[1]], [0.3, 0.12, 0.12], [STEEL; 3], Form::Round(Axis::X, 1.0));
    }

    /// The casing flange on the pad, the tubing head with its flow line, and the stuffing box the rod runs into.
    fn wellhead(&mut self) {
        let z = PIVOT[1] + HEAD;
        self.fixed([0.0, -1.36, z], [0.3, 0.08, 0.3], [IRON; 3], UPRIGHT);
        self.fixed([0.0, -1.1, z], [0.14, 0.44, 0.14], [STEEL; 3], UPRIGHT);
        self.fixed([0.12, -1.06, z], [0.24, 0.09, 0.09], [STEEL; 3], Form::Round(Axis::X, 1.0));
        self.fixed([0.0, -0.8, z], [0.11, 0.16, 0.11], [tex::MAT_BRASS; 3], UPRIGHT);
    }

    /// Each side's crank arm, its counterweight (two blocks, a sector's outline) and wrist pin, then the pitman arm up
    /// to the equalizer.
    fn crank(&mut self, pose: Pose) {
        let angle = (pose.pin[0] - CRANK[0]).atan2(pose.pin[1] - CRANK[1]);
        let along = |r: f64| [CRANK[0] + r * angle.sin(), CRANK[1] + r * angle.cos()];
        for x in [-1.0, 1.0] {
            self.fixed([0.28 * x, CRANK[0], CRANK[1]], [0.08, 0.16, 0.16], [IRON; 3], Form::Round(Axis::X, 1.0));
            self.tilted(at(0.28 * x, along(0.11)), angle, [0.06, 0.42, 0.14], IRON, false);
            // As big as the swing allows: the outer corners pass about 2 cm over the gearbox plinth.
            self.tilted(at(0.28 * x, along(0.33)), angle, [0.1, 0.1, 0.4], tex::STEEL, false);
            self.tilted(at(0.28 * x, along(0.41)), angle, [0.12, 0.12, 0.54], tex::STEEL, false);
            self.fixed(at(0.34 * x, pose.pin), [0.14, 0.07, 0.07], [STEEL; 3], Form::Round(Axis::X, 1.0));
            self.bar(at(0.4 * x, pose.pin), at(0.4 * x, pose.tail), [0.06, 0.06], GREEN);
        }
    }

    /// The walking beam (an I-beam), its tail bearing and equalizer, the horsehead, the bridle, the carrier bar and
    /// the polished rod.
    fn beam(&mut self, pose: Pose) {
        let t = pose.beam;
        // A point `u` along the beam from the bearing and `v` above its line.
        let on = |u: f64, v: f64| [PIVOT[0] + u * t.sin() + v * t.cos(), PIVOT[1] + u * t.cos() - v * t.sin()];
        self.tilted(at(0.0, on(-0.13, 0.12)), t, [0.06, 1.5, 0.17], GREEN, false);
        for v in [0.02, 0.22] {
            self.tilted(at(0.0, on(-0.13, v)), t, [0.18, 1.5, 0.03], GREEN, false);
        }
        self.tilted(at(0.0, on(-TAIL, 0.0)), t, [0.16, 0.12, 0.1], IRON, false);
        self.fixed(at(0.0, pose.tail), [0.84, 0.09, 0.09], [IRON; 3], Form::Round(Axis::X, 1.0));
        // The horsehead: a fan of wedges whose outer faces follow the arc, a dark groove plate on each for the bridle.
        const WEDGES: usize = 6;
        let step = (HEAD_ARC[1] - HEAD_ARC[0]) / WEDGES as f64;
        for k in 0..WEDGES {
            let a = HEAD_ARC[0] + step * (k as f64 + 0.5);
            let ray = |r: f64| on(r * a.cos(), r * a.sin());
            let arc = (HEAD * step) as f32;
            self.tilted(at(0.0, ray(HEAD - 0.17)), t + a, [0.2, 0.3, arc * 1.04], GREEN, false);
            let plate = [0.12, 0.04, arc * 1.02];
            self.tilted(at(0.0, ray(HEAD - 0.02)), t + a, plate, IRON, false);
        }
        // The bridle leaves the arc where it faces straight ahead, so it hangs plumb; the rod rises by the arc unwound.
        let (z, carrier) = (PIVOT[1] + HEAD + 0.012, PIVOT[0] - BRIDLE + HEAD * t);
        for x in [-0.045, 0.045] {
            self.bar([x, PIVOT[0], z], [x, carrier, z], [0.025, 0.025], tex::MAT_RUBBER);
        }
        self.fixed([0.0, carrier, z], [0.16, 0.05, 0.08], [STEEL; 3], Form::Box);
        self.fixed([0.0, carrier + 0.055, z], [0.06, 0.06, 0.06], [IRON; 3], Form::Box);
        self.fixed([0.0, carrier - ROD / 2.0, z], [0.035, ROD as f32, 0.035], [STEEL; 3], UPRIGHT);
    }

    /// A part that keeps the machine's own axes.
    fn fixed(&mut self, at: [f64; 3], size: [f32; 3], layers: [u16; 3], form: Form) {
        let size = size.map(|s| s * self.scale as f32);
        push_shape(self.out, form, self.point(at), self.yaw as f32, size, layers, false);
    }

    /// A box turned `angle` about the machine's x, its own y then along (z, y) = (cos, sin) of it: `size` is across,
    /// along and thick.
    fn tilted(&mut self, at: [f64; 3], angle: f64, size: [f32; 3], layer: u16, round: bool) {
        let (s, c) = angle.sin_cos();
        let axes = [self.dir([1.0, 0.0, 0.0]), self.dir([0.0, s, c]), self.dir([0.0, -c, s])];
        let look = Framed { taper: round.then_some(1.0), slide: None };
        let size = size.map(|v| v * self.scale as f32);
        push_framed(self.out, self.point(at), axes, size, [layer; 3], look);
    }

    /// A round bar from `a` to `b`, `thick` across.
    fn bar(&mut self, a: [f64; 3], b: [f64; 3], thick: [f32; 2], layer: u16) {
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if len < 1e-6 {
            return;
        }
        let y = d.map(|v| v / len);
        // Its own x lies level (the machine's x for an upright bar), z completes the frame.
        let flat = (y[0] * y[0] + y[2] * y[2]).sqrt();
        let x = if flat < 1e-6 { [1.0, 0.0, 0.0] } else { [y[2] / flat, 0.0, -y[0] / flat] };
        let z = [x[1] * y[2] - x[2] * y[1], x[2] * y[0] - x[0] * y[2], x[0] * y[1] - x[1] * y[0]];
        let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0];
        let size = [thick[0], len as f32, thick[1]].map(|v| v * self.scale as f32);
        let look = Framed { taper: Some(1.0), slide: None };
        push_framed(self.out, self.point(mid), [self.dir(x), self.dir(y), self.dir(z)], size, [layer; 3], look);
    }

    /// A model point, camera-relative.
    fn point(&self, p: [f64; 3]) -> Vec3 {
        self.centre + self.dir(p.map(|v| v * self.scale))
    }

    /// A model direction turned with the machine (as `model::local`).
    fn dir(&self, [x, y, z]: [f64; 3]) -> Vec3 {
        let (s, c) = self.yaw.sin_cos();
        Vec3::new(c * x - s * z, y, s * x + c * z)
    }
}

/// The model point at `x` across and side-view (y, z).
fn at(x: f64, [y, z]: [f64; 2]) -> [f64; 3] {
    [x, y, z]
}
