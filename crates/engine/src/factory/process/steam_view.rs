//! What a boiler and a turbine show besides their bodies (`BOILER_PARTS`, `TURBINE_PARTS`; drawn by `model::draw`,
//! presentation only). Neither has generic port hatches, so each connection says what it is:
//!
//! - a **coal chute** (a dark funnel with coal on top) at every belt inlet of a boiler: belts pointing into it
//!   bring fuel;
//! - a **water inlet** (a blue banded nipple on a flange) at every water port of a boiler;
//! - a **steam outlet** (a bigger, red banded nipple) at the boiler's two front ports and a steam inlet of the
//!   same make at the turbine's right end. A pipe piece on one meets it (`pipes.rs` draws its arm to the flange);
//! - a **water gauge** on the boiler's front, filled in proportion to the water it holds.
//!
//! The ports themselves are in `specs.rs`. To change the look: the constants below.

use crate::block::tex;
use crate::math::{IVec3, Vec3};

use super::super::footprint::Role;
use super::super::render::push_box;
use super::super::DIRS;
use super::model::local;
use super::steam::{UNIT_ENERGY, WATER_LOW};
use super::Processor;

/// The nipple's and the flange's width across, and how far from the cell's centre the nipple ends.
const WATER_WIDE: f32 = 0.3;
const STEAM_WIDE: f32 = 0.36;
const FLANGE_WIDE: f32 = 0.5;
const NIPPLE_END: f64 = 0.58;
/// Where the cell's wall is, and the flange's thickness.
const WALL: f64 = 0.4;
const FLANGE_THICK: f32 = 0.08;
/// The gauge: its place on the front, its centre height, its height and the most water it shows.
const GAUGE_X: f64 = 0.72;
const GAUGE_Y: f64 = 0.25;
const GAUGE_TALL: f32 = 0.7;
const GAUGE_HIGH: f64 = 0.6;
/// Water held at which the gauge is full: a unit just taken on top of the level that asks for one.
const FULL: u32 = WATER_LOW + UNIT_ENERGY;

/// Draws the connections of boiler `p`; `centre` is its footprint's centre relative to the camera, `rel` its
/// anchor cell's and `yaw` its turn.
pub(super) fn draw_boiler(p: &Processor, out: &mut Vec<f32>, rel: Vec3, centre: Vec3, yaw: f32) {
    let coal = [tex::COAL_ORE, tex::GENERATOR_SIDE, tex::FRAME];
    for (cell, side) in p.spec.footprint.faces(p.pos, p.dir, Role::In) {
        let at = rel + (cell - p.pos).as_vec3();
        let turn = ((side + 2) % 4) as f32 * std::f32::consts::FRAC_PI_2;
        let out_by = |d: f64, up: f64| at + DIRS[side as usize].as_vec3() * d + Vec3::new(0.0, up, 0.0);
        push_box(
            out,
            out_by(0.55, -0.04),
            turn,
            [0.6, 0.46, 0.3],
            0.0,
            [tex::FRAME, tex::GENERATOR_SIDE, tex::FRAME],
            false,
        );
        push_box(out, out_by(0.62, 0.22), turn, [0.82, 0.1, 0.44], 0.0, coal, false);
    }
    fittings(p, out, rel, Role::Water, tex::PIPE_WATER, WATER_WIDE);
    fittings(p, out, rel, Role::Steam, tex::PIPE_STEAM, STEAM_WIDE);
    let level = f64::from(p.steam.water.min(FULL)) / f64::from(FULL);
    let top = GAUGE_Y - f64::from(GAUGE_TALL) / 2.0;
    let frame = [tex::FRAME; 3];
    push_box(out, local(centre, yaw, [GAUGE_X, GAUGE_Y, 0.93]), yaw, [0.16, GAUGE_TALL, 0.08], 0.0, frame, false);
    let high = GAUGE_HIGH * level;
    if high > 0.02 {
        let y = top + 0.05 + high * 0.5;
        let size = [0.08, high as f32, 0.04];
        push_box(out, local(centre, yaw, [GAUGE_X, y, 0.97]), yaw, size, 0.0, [tex::WATER; 3], false);
    }
}

/// Draws the steam inlets of turbine `p`; `rel` is its anchor cell's centre relative to the camera.
pub(super) fn draw_turbine(p: &Processor, out: &mut Vec<f32>, rel: Vec3) {
    fittings(p, out, rel, Role::Steam, tex::PIPE_STEAM, STEAM_WIDE);
}

/// A nipple on a flange at every port of `role`, in `look`.
fn fittings(p: &Processor, out: &mut Vec<f32>, rel: Vec3, role: Role, look: u16, wide: f32) {
    for (cell, side, _) in p.pipe_ports().into_iter().filter(|&(_, _, r)| r == role) {
        fitting(out, rel + (cell - p.pos).as_vec3(), DIRS[side as usize], look, wide);
    }
}

/// One fitting on the wall of the cell at `at`, facing `d`: a nipple of `look` ending short of the next cell's
/// middle, and a steel flange where it leaves the wall.
fn fitting(out: &mut Vec<f32>, at: Vec3, d: IVec3, look: u16, wide: f32) {
    let boxed = |across: f32, from: f64, to: f64| {
        let thick = (to - from) as f32;
        (at + d.as_vec3() * ((from + to) / 2.0), [d.x, d.y, d.z].map(|a| if a == 0 { across } else { thick }))
    };
    let (nipple, size) = boxed(wide, WALL, NIPPLE_END);
    push_box(out, nipple, 0.0, size, 0.0, [look; 3], false);
    let (flange, size) = boxed(FLANGE_WIDE, WALL, WALL + f64::from(FLANGE_THICK));
    push_box(out, flange, 0.0, size, 0.0, [tex::STEEL; 3], false);
}
