//! 3D box/cylinder models for pipework (pipes, pumps and outlets; presentation only).
//!
//! - Pipes: round tubes (blue banded for water, pale/red for steam) with steel joint flanges on the positive face only
//!   so neighbours never z-fight.
//! - Pumps: skid with tier stripe/studs, round pump casing, motor housing, spinning cooling fan and status lamp.
//! - Outlets: round cast-iron valve body, bolted horizontal collar, curved downward-pointing brass discharge nozzle,
//!   top pressure gauge + status lamp, and a dynamic multi-segment parabolic water jet whose thickness, forward throw
//!   and spray scale visually with hydraulic pressure (`Pipework::pressure`).

use crate::block::tex;
use crate::math::Vec3;

use super::pipes::{Flow, Fluid, Part, Pipework};
use super::render::{push_box, push_framed, push_round, push_shape, Axis, Form, Framed};
use super::{tier_mark, DIRS, FACES};

/// A pipe's diameter, a junction's hub, and the flange round each joint (as fractions of a block).
const PIPE_D: f32 = 0.24;
const HUB_D: f32 = 0.34;
const FLANGE_D: f32 = 0.32;
const FLANGE_LEN: f32 = 0.06;
/// The round body's axis for world axis x, y, z.
const AXES: [Axis; 3] = [Axis::X, Axis::Up, Axis::Z];

impl Pipework {
    /// Camera- and time-independent pipework geometry (pipes, hubs, arms, flanges, pump body, outlet spout).
    pub(super) fn write_static(&self, out: &mut Vec<f32>, rel: Vec3) {
        let steel = [tex::STEEL; 3];
        let pipe = [match self.fluid {
            Fluid::Water => tex::PIPE_WATER,
            Fluid::Steam => tex::PIPE_STEAM,
            Fluid::Mixed => tex::LAMP_RED,
        }; 3];
        let joined = |f: usize| self.arms & (1 << f) != 0;
        let facing = if DIRS[self.facing as usize].x != 0 { 0 } else { 2 };
        let line = (0..6).filter(|&f| joined(f)).map(axis_of).try_fold(None, |line, a| match line {
            Some(l) if l != a => Err(()),
            _ => Ok(Some(a)),
        });
        let straight = match line {
            Ok(a) => Some(a.unwrap_or(facing)),
            Err(()) => None,
        };
        match (self.part, straight) {
            (Part::Pipe, Some(a)) => push_round(out, rel, 0.0, AXES[a], along(a, 1.0, PIPE_D), 1.0, pipe, false),
            (part, _) => {
                if part == Part::Pipe {
                    let arms_on = |a: usize| (0..6).filter(|&f| joined(f) && axis_of(f) == a).count();
                    let a = (0..3).max_by_key(|&a| arms_on(a)).unwrap_or(0);
                    push_round(out, rel, 0.0, AXES[a], along(a, HUB_D, HUB_D), 1.0, steel, false);
                }
                for (f, &n) in FACES.iter().enumerate().filter(|&(f, _)| joined(f)) {
                    let a = axis_of(f);
                    push_round(out, rel + n.as_vec3() * 0.25, 0.0, AXES[a], along(a, 0.5, PIPE_D), 1.0, pipe, false);
                }
            }
        }
        for (f, &n) in FACES.iter().enumerate().filter(|&(f, n)| joined(f) && n.x + n.y + n.z > 0) {
            let a = axis_of(f);
            push_round(out, rel + n.as_vec3() * 0.5, 0.0, AXES[a], along(a, FLANGE_LEN, FLANGE_D), 1.0, steel, false);
        }
        match self.part {
            Part::Pipe => {}
            Part::Pump => {
                let skid = if self.tier > 0 { tex::stripe(self.tier) } else { tex::FRAME };
                push_box(out, rel + Vec3::new(0.0, -0.425, 0.0), 0.0, [0.86, 0.13, 0.86], 0.0, [skid; 3], false);
                let casing = [tex::FRAME, tex::PIPE_WATER, tex::FRAME];
                push_round(out, rel + Vec3::new(0.0, -0.06, 0.0), 0.0, Axis::X, [0.6, 0.62, 0.62], 1.0, casing, false);
                let motor = [tex::FRAME, tex::MOTOR, tex::FRAME];
                push_box(out, rel + Vec3::new(0.0, 0.33, 0.0), 0.0, [0.34, 0.2, 0.34], 0.0, motor, false);
            }
            Part::Outlet => draw_outlet_static(out, rel, self.facing),
        }
    }

    /// Animated and near-camera parts of a pump or outlet (fan, pressure jet, status lamp, tier studs).
    pub(super) fn write_dynamic(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        match self.part {
            Part::Pipe => {}
            Part::Pump => {
                tier_mark::ring(out, rel + Vec3::new(0.0, -0.43, 0.0), 0.0, [0.43, 0.43], self.tier);
                let speed = 4.0 + 6.0 * (f64::from(self.pressure) / 1000.0);
                let spin = if self.flow == Flow::Working { (time * speed) as f32 } else { 0.0 };
                push_box(out, rel + Vec3::new(0.0, 0.45, 0.0), spin, [0.32, 0.04, 0.08], 0.0, [tex::STEEL; 3], false);
                push_box(
                    out,
                    rel + Vec3::new(0.0, 0.37, 0.18),
                    0.0,
                    [0.12, 0.08, 0.02],
                    0.0,
                    [lamp(self.flow); 3],
                    false,
                );
            }
            Part::Outlet => draw_outlet_dynamic(out, rel, self.facing, self.flow, self.pressure, time),
        }
    }
}

/// Intuitive 3D outlet body: round cast-iron valve housing, bolted discharge collar, horizontal pipe neck,
/// curved elbow bend, downward-flared brass spout nozzle, and top pressure gauge bezel.
fn draw_outlet_static(out: &mut Vec<f32>, rel: Vec3, facing: u8) {
    let d = DIRS[facing as usize].as_vec3();
    let horiz_axis = if DIRS[facing as usize].x != 0 { 0 } else { 2 };
    let cross_axis = if horiz_axis == 0 { Axis::Z } else { Axis::X };
    let yaw = d.x.atan2(d.z) as f32;
    let iron = [tex::MAT_GUNMETAL; 3];
    let steel = [tex::STEEL; 3];
    let brass = [tex::MAT_BRASS; 3];
    let water_pipe = [tex::PIPE_WATER; 3];

    // Central round cast-iron valve housing (radius 0.21, y = -0.20 .. +0.20).
    push_round(out, rel, 0.0, Axis::Up, [0.42, 0.40, 0.42], 1.0, [tex::STEEL, tex::MAT_GUNMETAL, tex::STEEL], false);
    // Horizontal discharge neck pointing cleanly toward `facing` (spans 0.05..0.39, clear of pipe arm caps at 0/0.5).
    push_round(out, rel + d * 0.22, 0.0, AXES[horiz_axis], along(horiz_axis, 0.34, 0.26), 1.0, water_pipe, false);
    // Bolted steel flange collar around the neck.
    push_round(out, rel + d * 0.23, 0.0, AXES[horiz_axis], along(horiz_axis, 0.07, 0.34), 1.0, steel, false);
    // Rounded cast-iron gooseneck elbow at the front edge.
    let elbow = rel + d * 0.39 + Vec3::new(0.0, -0.02, 0.0);
    push_round(out, elbow, 0.0, cross_axis, [0.27, 0.27, 0.27], 1.0, iron, false);
    // Angled brass discharge spout curving outward and downward toward the water exit.
    let spout = rel + d * 0.44 + Vec3::new(0.0, -0.15, 0.0);
    let forward = d;
    let down_tilt = norm(Vec3::new(0.0, -0.82, 0.0) + forward * 0.57);
    let right = Vec3::new(forward.z, 0.0, -forward.x);
    let back = cross(right, down_tilt);
    push_framed(
        out,
        spout,
        [right, down_tilt, back],
        [0.23, 0.26, 0.23],
        brass,
        Framed { taper: Some(1.18), slide: None },
    );
    // Top pressure gauge housing seated on the valve body (y = +0.20 .. +0.30; wider than 0.24 pipe arm).
    push_shape(out, Form::Round(Axis::Up, 1.0), rel + Vec3::new(0.0, 0.25, 0.0), yaw, [0.30, 0.10, 0.30], iron, false);
}

/// Dynamic outlet elements: status/pressure lamp on the top gauge, plus a pressure-driven parabolic water jet
/// arching out of the brass nozzle into the `front` voxel (and spray droplets at medium/high pressure).
fn draw_outlet_dynamic(out: &mut Vec<f32>, rel: Vec3, facing: u8, flow: Flow, pressure: u16, time: f64) {
    // Status lamp on top of the pressure gauge (y = +0.30 .. +0.38).
    push_box(out, rel + Vec3::new(0.0, 0.34, 0.0), 0.0, [0.12, 0.08, 0.12], 0.0, [lamp(flow); 3], false);
    if flow != Flow::Working || pressure == 0 {
        return;
    }
    let d = DIRS[facing as usize].as_vec3();
    let right = Vec3::new(d.z, 0.0, -d.x);
    let p = (f64::from(pressure) / 1000.0).clamp(0.2, 1.0);
    let width = (0.08 + 0.11 * p) as f32;
    let throw = 0.16 + 0.42 * p;
    let drop = 0.58;
    let scroll = (time * (2.0 + 3.0 * p)) as f32;
    let start = rel + d * 0.47 + Vec3::new(0.0, -0.22, 0.0);

    // Four parabolic jet segments connecting the brass spout mouth cleanly into the falling water voxel.
    const SEGS: usize = 4;
    for k in 0..SEGS {
        let t0 = k as f64 / SEGS as f64;
        let t1 = (k + 1) as f64 / SEGS as f64;
        let pt = |t: f64| start + d * (throw * t) - Vec3::new(0.0, 0.08 * t + drop * t * t, 0.0);
        let (a, b) = (pt(t0), pt(t1));
        let seg = b - a;
        let len = (seg.x * seg.x + seg.y * seg.y + seg.z * seg.z).sqrt();
        if len < 1e-4 {
            continue;
        }
        let mid = (a + b) * 0.5;
        let y_ax = seg * (1.0 / len);
        let z_ax = cross(right, y_ax);
        let w = width * (1.0 - 0.06 * k as f32);
        push_framed(
            out,
            mid,
            [right, y_ax, z_ax],
            [w, len as f32 * 1.08, w],
            [tex::WATER; 3],
            Framed { taper: Some(1.0), slide: Some(scroll) },
        );
    }

    // Animated white foam spray around the jet at medium/high pressure.
    if p >= 0.45 {
        for k in 0..3usize {
            let phase = (time * (2.2 + p) + k as f64 / 3.0).fract();
            let lateral = ((k as f64 - 1.0) * 0.10) * (0.4 + phase);
            let pos = start + d * (throw * (0.2 + 0.85 * phase)) + right * lateral
                - Vec3::new(0.0, 0.10 * phase + (drop + 0.18) * phase * phase, 0.0);
            let sz = ((0.08 + 0.05 * p) * (1.0 - 0.65 * phase)) as f32;
            push_box(out, pos, pos.x.atan2(pos.z) as f32, [sz; 3], 0.0, [tex::SMOKE_THIN; 3], false);
        }
    }
}

/// Unit vector along `v`.
fn norm(v: Vec3) -> Vec3 {
    let len = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    if len < 1e-9 {
        Vec3::ZERO
    } else {
        v * (1.0 / len)
    }
}

/// Right-handed cross product `a × b`.
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}

/// Draws a see-through placement preview of an outlet at `rel` facing `facing`, with connecting stubs for `arms`
/// and a preview water arc so the player immediately sees which way the spout will pour.
pub fn preview_outlet(out: &mut Vec<f32>, rel: Vec3, facing: u8, arms: u8, time: f64) {
    let mut p = Pipework::new(crate::math::IVec3::ZERO, crate::block::OUTLET, facing);
    p.arms = arms;
    p.flow = Flow::Working;
    p.pressure = 750;
    p.write_static(out, rel);
    p.write_dynamic(out, rel, time);
}

/// The world axis (0 x, 1 y, 2 z) of face `f` of `FACES`.
fn axis_of(f: usize) -> usize {
    let n = FACES[f];
    if n.x != 0 {
        0
    } else if n.y != 0 {
        1
    } else {
        2
    }
}

/// The size of a round body `len` long along world axis `a` and `d` across.
fn along(a: usize, len: f32, d: f32) -> [f32; 3] {
    let mut s = [d; 3];
    s[a] = len;
    s
}

/// The status lamp: green working, yellow full or blocked, red without power, water or head pressure.
fn lamp(flow: Flow) -> u16 {
    match flow {
        Flow::Working => tex::LAMP_GREEN,
        Flow::Full | Flow::Blocked => tex::LAMP_YELLOW,
        Flow::NoPower | Flow::NoWater | Flow::TooHigh => tex::LAMP_RED,
        Flow::Idle => tex::FRAME,
    }
}
