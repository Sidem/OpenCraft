//! The shape of a stretch of track: a smooth curve between two rail nodes, free of the voxel grid (only the nodes
//! sit on cells). Pure geometry, derived from the two nodes' cells and headings; nothing here is saved.
//!
//! The path is a cubic Hermite spline: in the horizontal plane it leaves node A along A's heading and arrives at B
//! along B's heading (each flipped to point the way of travel), with tangents as long as the chord, which makes a
//! straight chord straight and a mirrored pair of headings a near-perfect arc. Height changes linearly along the
//! parameter, so a ramp has an even grade. [`fit`] says whether a stretch is buildable (span, grade, turn radius).
//! To use a curve: [`Curve::new`], then `at`, `heading`, `length`; [`write_track`] draws it.

use crate::block::tex;
use crate::factory::render::push_box;
use crate::math::{IVec3, Vec3};

/// Headings are a byte: 256 steps round the compass, 0 facing north (-z) and 64 east (+x).
pub const YAW_STEPS: f64 = 256.0;
/// Longest span between two nodes (blocks, straight line) and shortest.
pub const MAX_SPAN: i32 = 32;
pub const MIN_SPAN: f64 = 3.0;
/// The tightest bend (blocks): a chord `L` leaving at `a` to its heading bends with radius `L / (2 sin a)`.
pub const MIN_RADIUS: f64 = 8.0;
/// The most a heading may differ from the chord, as the smallest allowed cosine (60 degrees).
const MIN_COS: f64 = 0.5;
/// Steepest grade: rise per block of horizontal run (one in three).
pub const MAX_GRADE: f64 = 1.0 / 3.0;

/// Rail geometry (block units, from the node's centre): the rails' height, their gap and thickness, a sleeper.
const RAIL_Y: f32 = -0.38;
const GAUGE: f32 = 0.18;
const RAIL_THICK: f32 = 0.06;
const SLEEPER_Y: f32 = -0.45;
/// Length of one drawn piece (a sleeper each), and the samples used to measure a curve.
const PIECE: f64 = 0.75;
const MEASURE: usize = 24;

/// Whether two nodes can be joined.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fit {
    Ok,
    /// Already joined.
    Joined,
    TooFar,
    /// Closer than `MIN_SPAN` (or straight above each other).
    TooClose,
    TooSteep,
    /// A heading points too far from the other node, or the bend is tighter than `MIN_RADIUS`.
    TooSharp,
    /// A node already holds `MAX_LINKS` tracks.
    Full,
}

/// The horizontal unit vector (x, z) a heading byte faces.
pub fn dir_of(yaw: u8) -> (f64, f64) {
    let a = f64::from(yaw) * std::f64::consts::TAU / YAW_STEPS;
    (a.sin(), -a.cos())
}

/// The heading byte nearest the horizontal direction (x, z); 0 for a zero vector.
pub fn yaw_of(x: f64, z: f64) -> u8 {
    let turns = x.atan2(-z) / std::f64::consts::TAU;
    (turns * YAW_STEPS).round().rem_euclid(YAW_STEPS) as u8
}

/// A path from node A to node B (world coordinates of the nodes' cell centres).
#[derive(Clone, Copy, Debug)]
pub struct Curve {
    a: Vec3,
    b: Vec3,
    /// Tangents in x and z, already as long as the chord.
    ma: (f64, f64),
    mb: (f64, f64),
}

impl Curve {
    /// The curve from the node at `a` facing `ya` to the node at `b` facing `yb`.
    pub fn new(a: IVec3, ya: u8, b: IVec3, yb: u8) -> Curve {
        let (a, b) = (a.as_vec3() + Vec3::new(0.5, 0.5, 0.5), b.as_vec3() + Vec3::new(0.5, 0.5, 0.5));
        let (cx, cz) = (b.x - a.x, b.z - a.z);
        let run = cx.hypot(cz);
        // A heading is an axis: point it the way of travel.
        let along = |yaw: u8| {
            let (x, z) = dir_of(yaw);
            let s = if x * cx + z * cz < 0.0 { -run } else { run };
            (x * s, z * s)
        };
        Curve { a, b, ma: along(ya), mb: along(yb) }
    }

    /// The point `t` (0 at A, 1 at B) along the curve.
    pub fn at(&self, t: f64) -> Vec3 {
        let (t2, t3) = (t * t, t * t * t);
        let (h00, h10, h01, h11) = (2.0 * t3 - 3.0 * t2 + 1.0, t3 - 2.0 * t2 + t, 3.0 * t2 - 2.0 * t3, t3 - t2);
        Vec3::new(
            h00 * self.a.x + h10 * self.ma.0 + h01 * self.b.x + h11 * self.mb.0,
            self.a.y + (self.b.y - self.a.y) * t,
            h00 * self.a.z + h10 * self.ma.1 + h01 * self.b.z + h11 * self.mb.1,
        )
    }

    /// The horizontal direction of travel at `t`, as a unit vector (x, z): what a train steers by.
    pub fn heading(&self, t: f64) -> (f64, f64) {
        let t2 = t * t;
        let (d00, d10, d01, d11) =
            (6.0 * t2 - 6.0 * t, 3.0 * t2 - 4.0 * t + 1.0, 6.0 * t - 6.0 * t2, 3.0 * t2 - 2.0 * t);
        let x = d00 * self.a.x + d10 * self.ma.0 + d01 * self.b.x + d11 * self.mb.0;
        let z = d00 * self.a.z + d10 * self.ma.1 + d01 * self.b.z + d11 * self.mb.1;
        let len = x.hypot(z).max(1e-9);
        (x / len, z / len)
    }

    /// The length of the path, in blocks.
    pub fn length(&self) -> f64 {
        let mut prev = self.at(0.0);
        (1..=MEASURE)
            .map(|i| {
                let p = self.at(i as f64 / MEASURE as f64);
                let d = p - prev;
                prev = p;
                d.length()
            })
            .sum()
    }
}

/// Whether nodes at `a` and `b` facing `ya` and `yb` can be joined by one stretch (the links they already
/// hold are the caller's to check, see `Fit::Joined` and `Fit::Full`).
pub fn fit(a: IVec3, ya: u8, b: IVec3, yb: u8) -> Fit {
    let d = (b - a).as_vec3();
    let (run, span) = (d.x.hypot(d.z), d.length());
    if span > f64::from(MAX_SPAN) {
        return Fit::TooFar;
    }
    if span < MIN_SPAN || run < 1.0 {
        return Fit::TooClose;
    }
    if d.y.abs() > run * MAX_GRADE + 1e-9 {
        return Fit::TooSteep;
    }
    for yaw in [ya, yb] {
        let (x, z) = dir_of(yaw);
        let cos = ((x * d.x + z * d.z) / run).abs();
        let sin = ((x * d.z - z * d.x) / run).abs();
        // Radius run / (2 sin) must not be tighter than MIN_RADIUS.
        if cos < MIN_COS || sin * 2.0 * MIN_RADIUS > run {
            return Fit::TooSharp;
        }
    }
    Fit::Ok
}

/// A node's model at camera-relative `rel` (its cell's centre): a steel plate under the track and, while no
/// track ends here (`bare`), a short stub of rails showing the heading.
pub fn write_node(out: &mut Vec<f32>, rel: Vec3, yaw: u8, bare: bool) {
    let a = (f64::from(yaw) * std::f64::consts::TAU / YAW_STEPS) as f32;
    push_box(out, rel + Vec3::new(0.0, -0.47, 0.0), a, [0.8, 0.05, 0.8], 0.0, [tex::FRAME; 3], false);
    if bare {
        let (s, c) = a.sin_cos();
        for x in [-GAUGE, GAUGE] {
            let at = rel + Vec3::new(f64::from(c * x), f64::from(RAIL_Y), f64::from(s * x));
            push_box(out, at, a, [RAIL_THICK, 0.08, 1.2], 0.0, [tex::STEEL; 3], false);
        }
        push_box(
            out,
            rel + Vec3::new(0.0, f64::from(SLEEPER_Y), 0.0),
            a,
            [0.5, 0.05, 0.14],
            0.0,
            [tex::PLANKS; 3],
            false,
        );
    }
}

/// A signal beside the node at `rel`: a steel post a block to the side of the track with an amber lamp on top.
pub fn write_signal(out: &mut Vec<f32>, rel: Vec3, yaw: u8) {
    let a = (f64::from(yaw) * std::f64::consts::TAU / YAW_STEPS) as f32;
    let (s, c) = a.sin_cos();
    let side = rel + Vec3::new(f64::from(c), 0.0, f64::from(s));
    push_box(out, side + Vec3::new(0.0, 0.05, 0.0), a, [0.14, 1.0, 0.14], 0.0, [tex::STEEL; 3], false);
    push_box(out, side + Vec3::new(0.0, 0.65, 0.0), a, [0.34, 0.3, 0.34], 0.0, [tex::DOCK_LOAD_TOP; 3], false);
}

/// Draws the track of `curve` as short pieces, each a sleeper and two steel rails, for the pieces within `range` of
/// the camera at `eye`.
pub fn write_track(out: &mut Vec<f32>, curve: &Curve, eye: Vec3, range: f64) {
    let n = ((curve.length() / PIECE).ceil() as usize).max(2);
    let point = |i: usize| curve.at(i as f64 / n as f64) - eye;
    let mut from = point(0);
    for i in 0..n {
        let to = point(i + 1);
        let (mid, d) = ((from + to) * 0.5, to - from);
        if mid.length() <= range {
            let run = d.x.hypot(d.z).max(1e-9);
            let yaw = d.x.atan2(-d.z) as f32;
            let pitch = d.y.atan2(run) as f32;
            let len = d.length() as f32;
            let (s, c) = yaw.sin_cos();
            let side = |x: f32, y: f32| mid + Vec3::new(f64::from(c * x), f64::from(y), f64::from(s * x));
            let mut piece = |x: f32, y: f32, size: [f32; 3], layer: u16| {
                push_box(out, side(x, y), yaw, size, 0.0, [layer; 3], false);
                let end = out.len();
                out[end - 4] = pitch;
            };
            for x in [-GAUGE, GAUGE] {
                piece(x, RAIL_Y, [RAIL_THICK, 0.08, len + 0.03], tex::STEEL);
            }
            piece(0.0, SLEEPER_Y, [0.5, 0.05, 0.14], tex::PLANKS);
        }
        from = to;
    }
}
