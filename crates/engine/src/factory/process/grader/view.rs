//! How the Telescoping Bucket-Wheel Site Grader looks (presentation only).
//!
//! Uses dedicated industrial pixel-art textures (`textures/grader.rs`): weathered ochre-yellow steel plating
//! (`GRADER_YELLOW`), louvered diesel cowling (`GRADER_LOUVRE`), forged crawler grouser treads (`GRADER_TREAD`),
//! box-girder boom trusses (`GRADER_TRUSS`), and scuffed burnt-orange bucket steel (`GRADER_BUCKET`).
//!
//! Static undercarriage (`PARTS`): twin tracked crawlers with rounded front/rear steel drive sprockets and yellow
//! bogie rails, gunmetal carbody deck, left paving intake hopper, rear spoil chute, circular slewing bearing drum,
//! and status lamp.
//!
//! Dynamic superstructure (`draw_grader`): slewing turret house with glazed cab & twin exhaust stacks, concrete &
//! hazard counterweight, live earthworks hopper gauge, steel A-frame cable gantry tower, optimized 3-stage telescoping
//! truss boom with return conveyor & lift rams, continuously spinning 8-bucket toothed cutting wheel, and clean site
//! perimeter survey beacons & grade-plane guide rails.

use std::f64::consts::TAU;

use crate::block::tex;
use crate::math::{IVec3, Vec3};
use crate::TICK;

use super::super::super::power::FULL_SPEED;
use super::super::super::render::{push_box, push_framed, Framed};
use super::super::super::{ticks, DIRS};
use super::super::model::{part, round, round_x, Look, Part};
use super::super::{Extra, Processor, Status};
use super::{Grader, ACT_FELL, ACT_FILL, ACT_IDLE, ACT_PAVE, FELL_SECONDS, FILL_CAP, WORK_SECONDS};

const HULL: u16 = tex::GRADER_YELLOW;
const LOUVRE: u16 = tex::GRADER_LOUVRE;
const TREAD: u16 = tex::GRADER_TREAD;
const TRUSS: u16 = tex::GRADER_TRUSS;
const BUCKET: u16 = tex::GRADER_BUCKET;
const DARK: u16 = tex::MAT_GUNMETAL;
const STEEL: u16 = tex::STEEL;
const SOOT: u16 = tex::SOOT;

/// Turret slew pivot in local coordinates (footprint is 3×3×2, so y runs -1.0..+1.0).
const PIVOT: [f32; 3] = [0.0, -0.04, -0.02];

/// Static crawler undercarriage & slewing pedestal (16 parts; verified zero coplanar overlapping faces).
pub(super) const PARTS: [Part; 16] = [
    // Twin forged-steel grouser crawler tracks (main straight runs).
    part([-1.14, -0.80, 0.0], [0.54, 0.40, 2.48], Look::Tex([TREAD, TREAD, TREAD])),
    part([1.14, -0.80, 0.0], [0.54, 0.40, 2.48], Look::Tex([TREAD, TREAD, TREAD])),
    // Rounded front & rear steel drive sprockets / idler drums at the track ends.
    round_x([-1.14, -0.80, -1.24], [0.50, 0.38, 0.38], Look::Tex([DARK, TREAD, DARK])),
    round_x([-1.14, -0.80, 1.24], [0.50, 0.38, 0.38], Look::Tex([DARK, TREAD, DARK])),
    round_x([1.14, -0.80, -1.24], [0.50, 0.38, 0.38], Look::Tex([DARK, TREAD, DARK])),
    round_x([1.14, -0.80, 1.24], [0.50, 0.38, 0.38], Look::Tex([DARK, TREAD, DARK])),
    // Weathered yellow outer track bogie beams.
    part([-1.44, -0.80, 0.0], [0.06, 0.20, 2.36], Look::Tex([HULL; 3])),
    part([1.44, -0.80, 0.0], [0.06, 0.20, 2.36], Look::Tex([HULL; 3])),
    // Main gunmetal carbody deck between tracks.
    part([0.0, -0.74, 0.0], [1.72, 0.36, 2.46], Look::Tex([DARK; 3])),
    // Upper turntable collar plate.
    part([0.0, -0.51, -0.02], [2.06, 0.10, 2.12], Look::Tex([HULL, HULL, DARK])),
    // Rear spoil discharge chute (facing +z = Side::Front output port).
    part([0.0, -0.72, 1.32], [0.84, 0.28, 0.18], Look::Tex([STEEL, DARK, DARK])),
    // Left floor-material intake chute (facing -x = Side::Left input port).
    part([-1.24, -0.49, 0.0], [0.36, 0.20, 0.86], Look::Tex([HULL, STEEL, DARK])),
    // Circular slewing pedestal drum & machined steel bearing ring.
    round([0.0, -0.31, -0.02], [1.56, 0.30, 1.56], Look::Tex([DARK; 3])),
    round([0.0, -0.10, -0.02], [1.28, 0.12, 1.28], Look::Tex([STEEL; 3])),
    // Status lamp on front-right deck.
    part([0.82, -0.38, 0.94], [0.16, 0.16, 0.12], Look::Lamp),
    // Dynamic turret, telescoping boom, spinning bucket wheel and perimeter guide rails.
    part(PIVOT, [0.1, 0.1, 0.1], Look::Grader),
];

/// Draws the dynamic slewing superstructure, 3-stage telescoping boom, spinning 8-bucket wheel and site perimeter.
pub(in crate::factory::process) fn draw_grader(p: &Processor, out: &mut Vec<f32>, rel: Vec3, pivot: Vec3, time: f64) {
    let Extra::Grader(g) = &p.extra else { return };
    let working = p.status == Status::Working && g.active != ACT_IDLE;

    // Draw site perimeter survey beacons & grade-plane guide rails.
    draw_perimeter(out, p, g, rel);

    // Smoothly interpolate the active target cell from `prev` to `cur`.
    let site_fwd = DIRS[p.dir as usize % 4].as_vec3();
    let target = if g.active != ACT_IDLE && g.cur != IVec3::ZERO {
        let s = step_share(p, g, time);
        let move_t = (s * 2.2).clamp(0.0, 1.0);
        let ease = move_t * move_t * (3.0 - 2.0 * move_t);
        let a = rel + (g.prev - p.pos).as_vec3();
        let b = rel + (g.cur - p.pos).as_vec3();
        let jump_lift = if (b - a).length() > 1.5 { (move_t * std::f64::consts::PI).sin() * 0.65 } else { 0.0 };
        a + (b - a) * ease + up(jump_lift)
    } else {
        pivot + site_fwd * 7.5 + up(1.4)
    };

    let flat = Vec3::new(target.x - pivot.x, 0.0, target.z - pivot.z);
    let fwd = if flat.length() > 0.25 { unit(flat) } else { site_fwd };
    let right = unit(cross(fwd, up(1.0)));

    let turret = TurretFrame { origin: pivot, right, up: up(1.0), fwd };
    draw_turret(out, &turret, g, working, time);

    let wheel_r = 1.15;
    let boom_root = turret.at(0.0, 0.88, 0.68);
    let wheel_hub = target - fwd * (wheel_r * 0.72) + up(wheel_r * 0.62);
    let gantry_top = turret.at(0.0, 3.60, -0.35);
    draw_telescoping_boom(out, &turret, boom_root, wheel_hub, gantry_top, working, time);

    let spin = if working { time * 4.2 } else { 0.35 };
    draw_bucket_wheel(out, wheel_hub, fwd, right, wheel_r, spin, g.active, working, time);
}

/// Smooth progress `0..=1` through the current block cycle.
fn step_share(p: &Processor, g: &Grader, time: f64) -> f64 {
    let sec = if g.active == ACT_FELL { FELL_SECONDS } else { WORK_SECONDS };
    let goal = f64::from(ticks(sec) * FULL_SPEED);
    let rate = f64::from(p.stats().speed) * f64::from(p.speed) / 1e3 * f64::from(p.boost) / 1e3;
    let ahead = if p.status == Status::Working { (time / TICK).fract() * rate } else { 0.0 };
    ((f64::from(p.progress) + ahead) / goal.max(1.0)).clamp(0.0, 1.0)
}

struct TurretFrame {
    origin: Vec3,
    right: Vec3,
    up: Vec3,
    fwd: Vec3,
}

impl TurretFrame {
    fn at(&self, r: f64, u: f64, f: f64) -> Vec3 {
        self.origin + self.right * r + self.up * u + self.fwd * f
    }

    fn box_part(&self, out: &mut Vec<f32>, c: [f64; 3], size: [f32; 3], layers: [u16; 3], round: bool) {
        let look = Framed { taper: round.then_some(1.0), slide: None };
        push_framed(out, self.at(c[0], c[1], c[2]), [self.right, self.up, self.fwd], size, layers, look);
    }
}

/// Draws the weathered yellow slewing house, louvered diesel engine, glazed cab, counterweight, hopper gauge & gantry.
fn draw_turret(out: &mut Vec<f32>, t: &TurretFrame, g: &Grader, working: bool, time: f64) {
    // Main slewing turntable deck & steel skirt.
    t.box_part(out, [0.0, 0.24, -0.15], [1.96, 0.46, 2.28], [HULL, HULL, DARK], false);
    // Louvered diesel engine housing + twin soot exhaust stacks on the left side.
    t.box_part(out, [-0.46, 0.72, -0.35], [0.86, 0.54, 1.54], [DARK, LOUVRE, DARK], false);
    t.box_part(out, [-0.62, 1.16, -0.78], [0.14, 0.38, 0.14], [SOOT; 3], true);
    t.box_part(out, [-0.36, 1.16, -0.78], [0.14, 0.38, 0.14], [SOOT; 3], true);

    // Glazed industrial operator cab on the right-front overlooking the boom.
    t.box_part(out, [0.58, 0.88, 0.36], [0.76, 0.80, 0.96], [HULL; 3], false);
    t.box_part(out, [0.58, 0.96, 0.85], [0.62, 0.46, 0.04], [tex::CAB_GLASS; 3], false);
    t.box_part(out, [0.97, 0.96, 0.36], [0.04, 0.46, 0.74], [tex::CAB_GLASS; 3], false);
    t.box_part(out, [0.58, 1.32, 0.40], [0.84, 0.08, 1.12], [DARK; 3], false);
    // Flashing amber warning beacon on cab roof.
    let beacon = if working && (time * 5.0).sin() > 0.0 { tex::LAMP_YELLOW } else { BUCKET };
    t.box_part(out, [0.58, 1.43, 0.36], [0.14, 0.14, 0.14], [beacon; 3], true);

    // Rear counterweight boom arms & heavy concrete/hazard ballast slab.
    beam(out, t.at(-0.54, 0.42, -1.05), t.at(-0.54, 0.62, -2.50), [0.20, 0.22], [TRUSS; 3], false);
    beam(out, t.at(0.54, 0.42, -1.05), t.at(0.54, 0.62, -2.50), [0.20, 0.22], [TRUSS; 3], false);
    t.box_part(out, [0.0, 0.66, -2.46], [1.72, 0.76, 0.80], [DARK, tex::CONCRETE, DARK], false);
    t.box_part(out, [0.0, 1.10, -2.46], [1.62, 0.14, 0.70], [BUCKET; 3], false);

    // Open cut-and-fill earthworks ballast hopper behind the cab: shows live earth fill level.
    t.box_part(out, [0.46, 0.66, -0.78], [0.76, 0.42, 0.84], [DARK, STEEL, DARK], false);
    let fill = g.hopper_total();
    if fill > 0 {
        let ratio = (f64::from(fill) / f64::from(FILL_CAP)).clamp(0.08, 1.0);
        let tex_fill = crate::block::def(g.hopper_top_block()).faces[0];
        let h = (0.12 + 0.40 * ratio) as f32;
        t.box_part(out, [0.46, 0.68 + f64::from(h) * 0.5, -0.78], [0.62, h, 0.68], [tex_fill; 3], false);
    }

    // Steel A-frame suspension mast / gantry tower.
    let top_l = t.at(-0.42, 3.60, -0.35);
    let top_r = t.at(0.42, 3.60, -0.35);
    beam(out, t.at(-0.76, 0.48, 0.40), top_l, [0.15, 0.17], [STEEL; 3], false);
    beam(out, t.at(0.76, 0.48, 0.40), top_r, [0.15, 0.17], [STEEL; 3], false);
    beam(out, t.at(-0.76, 0.48, -1.14), top_l, [0.15, 0.17], [STEEL; 3], false);
    beam(out, t.at(0.76, 0.48, -1.14), top_r, [0.15, 0.17], [STEEL; 3], false);
    beam(out, top_l - t.right * 0.14, top_r + t.right * 0.14, [0.22, 0.22], [BUCKET; 3], true);
    // Backstay tension cables tying gantry apex to counterweight.
    beam(out, top_l, t.at(-0.60, 1.06, -2.46), [0.045, 0.045], [SOOT; 3], true);
    beam(out, top_r, t.at(0.60, 1.06, -2.46), [0.045, 0.045], [SOOT; 3], true);
}

/// Draws the optimized 3-stage telescoping truss boom, return spoil belt, lift rams, and hoist cables.
fn draw_telescoping_boom(
    out: &mut Vec<f32>,
    t: &TurretFrame,
    root: Vec3,
    tip: Vec3,
    gantry_top: Vec3,
    working: bool,
    time: f64,
) {
    let delta = tip - root;
    let span = delta.length().max(2.0);
    let along = delta * (1.0 / span);
    let side = unit(cross(along, up(1.0)));
    let normal = cross(side, along);

    // Boom trunnion axle on turret front.
    beam(out, root - side * 0.44, root + side * 0.44, [0.26, 0.26], [BUCKET; 3], true);

    // 3 telescoping stages proportioned evenly across any reach (5..=41+ blocks).
    let s1_end = (span * 0.36).max(1.6);
    let s2_start = (s1_end - 0.5).max(0.4);
    let s2_end = (span * 0.70).max(s1_end + 0.5);
    let s3_start = (s2_end - 0.5).max(s2_start + 0.3);
    let yoke_base = tip - along * 0.55;

    // Stage 1: single unified riveted truss box-girder (cleaner & fewer instances).
    beam(out, root, root + along * s1_end, [0.56, 0.48], [HULL, TRUSS, DARK], false);
    // Stage 2: mid telescoping gunmetal box beam.
    beam(out, root + along * s2_start, root + along * s2_end, [0.38, 0.36], [DARK, STEEL, DARK], false);
    // Stage 3: inner telescoping machined steel beam.
    beam(out, root + along * s3_start, yoke_base, [0.24, 0.24], [STEEL; 3], false);

    // Return spoil conveyor belt along the top of the boom.
    let belt_start = root + normal * 0.28;
    let belt_end = yoke_base + normal * 0.22;
    beam(out, belt_start, belt_end, [0.28, 0.06], [tex::BELT_TOP, SOOT, DARK], false);

    if working {
        for k in 0..3 {
            let phase = 1.0 - (time * 0.45 + f64::from(k) * 0.33).fract();
            let clod_pos = belt_start + (belt_end - belt_start) * phase + normal * 0.11;
            push_box(out, clod_pos, 0.0, [0.19, 0.15, 0.19], 0.0, [tex::DIRT; 3], false);
        }
    }

    // Twin hydraulic lift rams under Stage 1.
    let ram_attach = root + along * (s1_end * 0.58) - normal * 0.20;
    for sgn in [-1.0, 1.0] {
        let base = t.at(sgn * 0.50, 0.18, 0.86);
        let head = ram_attach + side * (sgn * 0.28);
        let mid = base + (head - base) * 0.55;
        beam(out, base, mid, [0.13, 0.13], [DARK; 3], true);
        beam(out, mid, head, [0.075, 0.075], [STEEL; 3], true);
    }

    // Twin suspension hoist cables from A-frame apex to outer boom.
    let cable_anchor = root + along * (span * 0.84) + normal * 0.20;
    for sgn in [-1.0, 1.0] {
        beam(
            out,
            gantry_top + t.right * (sgn * 0.36),
            cable_anchor + side * (sgn * 0.22),
            [0.045, 0.045],
            [SOOT; 3],
            true,
        );
    }

    // Forked ochre-yellow wheel yoke embracing the axle.
    for sgn in [-1.0, 1.0] {
        beam(out, yoke_base + side * (sgn * 0.16), tip + side * (sgn * 0.42), [0.13, 0.18], [HULL; 3], false);
    }
}

/// Draws the optimized spinning 8-bucket toothed cutting wheel at `hub` with radius `r`.
#[allow(clippy::too_many_arguments)]
fn draw_bucket_wheel(
    out: &mut Vec<f32>,
    hub: Vec3,
    fwd: Vec3,
    right: Vec3,
    r: f64,
    spin: f64,
    active: u8,
    working: bool,
    time: f64,
) {
    // Central heavy axle & yellow gearbox hub.
    beam(out, hub - right * 0.48, hub + right * 0.48, [0.34, 0.34], [DARK; 3], true);
    beam(out, hub - right * 0.30, hub + right * 0.30, [0.54, 0.54], [HULL; 3], true);

    const SPOKES: usize = 8;
    let mut ring_pts = [Vec3::ZERO; SPOKES];
    let look = Framed { taper: None, slide: None };
    let bucket_tex = match active {
        ACT_PAVE => [tex::CONCRETE, BUCKET, DARK],
        ACT_FILL => [tex::DIRT, BUCKET, DARK],
        _ => [BUCKET; 3],
    };

    for (k, pt) in ring_pts.iter_mut().enumerate() {
        let a = spin + (k as f64) * TAU / (SPOKES as f64);
        let radial = fwd * a.cos() + up(a.sin());
        *pt = hub + radial * r;
        // Spoke from hub to rim.
        beam(out, hub + radial * 0.22, *pt, [0.11, 0.14], [STEEL; 3], false);

        // Tangential forged digging bucket + hardened steel tooth lip.
        let tangent = fwd * (-a.sin()) + up(a.cos());
        let scoop_c = hub + radial * (r + 0.16) + tangent * 0.08;
        push_framed(out, scoop_c, [right, radial, tangent], [0.56, 0.30, 0.34], bucket_tex, look);
        let teeth_c = hub + radial * (r + 0.26) + tangent * 0.26;
        push_framed(out, teeth_c, [right, radial, tangent], [0.52, 0.10, 0.10], [STEEL; 3], look);
    }

    // Unified heavy octagonal rim ring connecting the 8 spokes (8 clean segments instead of 16).
    for k in 0..SPOKES {
        beam(out, ring_pts[k], ring_pts[(k + 1) % SPOKES], [0.24, 0.16], [HULL, HULL, DARK], false);
    }

    if working {
        let bite = hub + fwd * (r * 0.45) - up(r * 0.85);
        for k in 0..3 {
            let t = (time * 3.4 + f64::from(k) * 0.33).fract();
            let side_j = ((k as f64 - 1.0) * 0.36) * t;
            let chip = bite + right * side_j - fwd * (t * 0.7) + up(t * 0.9 - t * t * 1.4);
            let tex_chip = if active == ACT_PAVE { tex::CONCRETE } else { tex::DIRT };
            push_box(out, chip, (time * 6.0 + f64::from(k)) as f32, [0.14; 3], 0.0, [tex_chip; 3], false);
        }
    }
}

/// Draws grounded industrial survey pylons at the 4 site corners and clean grade-plane sight rails along the pad edge.
fn draw_perimeter(out: &mut Vec<f32>, p: &Processor, g: &Grader, rel: Vec3) {
    let fwd = DIRS[p.dir as usize % 4].as_vec3();
    let right = DIRS[(p.dir as usize + 1) % 4].as_vec3();
    let origin = rel + right - up(0.5);
    let near = 2.5;
    let far = 2.5 + f64::from(g.depth);
    let (left_col, right_col) = g.across_bounds();
    let left = f64::from(left_col) - 0.5;
    let r_edge = f64::from(right_col) + 0.5;
    let pt = |a: f64, s: f64| origin + fwd * a + right * s;
    let corners = [pt(near, left), pt(far, left), pt(far, r_edge), pt(near, r_edge)];

    for c in corners {
        push_box(out, c + up(0.10), 0.0, [0.28, 0.20, 0.28], 0.0, [DARK; 3], false);
        push_box(out, c + up(0.62), 0.0, [0.14, 0.86, 0.14], 0.0, [DARK, HULL, DARK], false);
        push_box(out, c + up(1.14), 0.0, [0.20, 0.18, 0.20], 0.0, [tex::LAMP_YELLOW; 3], false);
    }

    for i in 0..4 {
        let a = corners[i] + up(0.35);
        let b = corners[(i + 1) % 4] + up(0.35);
        beam(out, a, b, [0.055, 0.055], [HULL; 3], false);
    }
}

fn beam(out: &mut Vec<f32>, a: Vec3, b: Vec3, thick: [f32; 2], layers: [u16; 3], round: bool) {
    let len = (b - a).length();
    if len < 1e-4 {
        return;
    }
    let y = (b - a) * (1.0 / len);
    let side = if y.y.abs() > 0.99 { Vec3::new(1.0, 0.0, 0.0) } else { unit(cross(y, up(1.0))) };
    let z = cross(side, y);
    let look = Framed { taper: round.then_some(1.0), slide: None };
    push_framed(out, a + (b - a) * 0.5, [side, y, z], [thick[0], len as f32, thick[1]], layers, look);
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}

fn unit(v: Vec3) -> Vec3 {
    v * (1.0 / v.length().max(1e-9))
}

fn up(h: f64) -> Vec3 {
    Vec3::new(0.0, h, 0.0)
}
