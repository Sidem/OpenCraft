//! Processor models as data: a spec's `parts` are boxes (centre and size in block units, relative to
//! the footprint's centre, front towards +z) turned with the machine, each with a `Look`. The tier band,
//! the status lamp, a fire that glows while working and a press that pumps are looks, so a new processor
//! is rows, not drawing code. A multi-block machine's port hatches are drawn from its footprint.

use crate::block::tex;
use crate::math::Vec3;

use super::super::footprint::Role;
use super::super::render::push_box;
use super::super::DIRS;
use super::{steam_view, Energy, Processor, Status};

/// How a part is textured, and whether it moves.
#[derive(Clone, Copy)]
pub enum Look {
    /// Top, side and bottom layers.
    Tex([u16; 3]),
    /// The tier band: its top layer, sides in the tier's stripe (`tex::stripe`).
    Band(u16),
    /// Lit while working, else this layer.
    Fire(u16),
    /// The status lamp: green working, yellow output full, red for a missing recipe, fuel or power.
    Lamp,
    /// Moves down by this share of the press stroke while working.
    Press(f32, [u16; 3]),
}

#[derive(Clone, Copy)]
pub struct Part {
    pub at: [f32; 3],
    pub size: [f32; 3],
    pub look: Look,
}

pub const fn part(at: [f32; 3], size: [f32; 3], look: Look) -> Part {
    Part { at, size, look }
}

/// Deepest press stroke, in blocks.
const STROKE: f64 = 0.12;
/// A port hatch: how far its centre sits from its cell's centre, and its size (across, up, thick).
const HATCH_OUT: f64 = 0.45;
const HATCH: [f32; 3] = [0.66, 0.5, 0.1];

/// Draws `p`'s parts at `rel` (its anchor cell's centre relative to the camera).
pub fn draw(p: &Processor, out: &mut Vec<f32>, rel: Vec3, time: f64) {
    let working = p.status == Status::Working;
    let stroke = if working { (time * 6.0).sin().abs() * STROKE } else { 0.0 };
    let yaw = p.dir as f32 * std::f32::consts::FRAC_PI_2;
    let centre = rel + p.spec.footprint.centre(p.dir);
    for part in p.spec.parts {
        let mut at = local(centre, yaw, part.at.map(f64::from));
        let texture = match part.look {
            Look::Tex(t) => t,
            Look::Band(top) => [top, tex::stripe(p.tier), tex::FRAME],
            Look::Fire(idle) => [if working { tex::LAMP_YELLOW } else { idle }; 3],
            Look::Lamp => [lamp(p.status); 3],
            Look::Press(share, t) => {
                at.y -= stroke * share as f64;
                t
            }
        };
        push_box(out, at, yaw, part.size, 0.0, texture, false);
    }
    match p.energy() {
        // Chutes, pipe fittings and a gauge instead of the generic hatches.
        Energy::Boiler => return steam_view::draw_boiler(p, out, rel, centre, yaw),
        Energy::Turbine => return steam_view::draw_turbine(p, out, rel),
        _ => {}
    }
    if p.spec.footprint.is_single() {
        return;
    }
    steam_view::fittings(p, out, rel, Role::Water, tex::PIPE_WATER, steam_view::WATER_WIDE);
    for (role, layer) in [(Role::In, tex::PORT_IN), (Role::Out, tex::PORT_OUT), (Role::Side, tex::PORT_SIDE)] {
        for (cell, side) in p.spec.footprint.faces(p.pos, p.dir, role) {
            let at =
                rel + (cell - p.pos).as_vec3() + DIRS[side as usize].as_vec3() * HATCH_OUT - Vec3::new(0.0, 0.12, 0.0);
            // Turned so its +z face (the hatch) looks out of `side`.
            let yaw = ((side + 2) % 4) as f32 * std::f32::consts::FRAC_PI_2;
            push_box(out, at, yaw, HATCH, 0.0, [tex::FRAME, layer, tex::FRAME], false);
        }
    }
}

/// A point `at` of a part, in the machine's own frame (front towards +z), turned by `yaw` about `centre`.
pub(super) fn local(centre: Vec3, yaw: f32, at: [f64; 3]) -> Vec3 {
    let (s, c) = (yaw.sin() as f64, yaw.cos() as f64);
    let [x, y, z] = at;
    centre + Vec3::new(c * x - s * z, y, s * x + c * z)
}

fn lamp(status: Status) -> u16 {
    match status {
        Status::Working => tex::LAMP_GREEN,
        Status::OutputFull => tex::LAMP_YELLOW,
        Status::NoRecipe | Status::NoPower | Status::NoFuel | Status::NoWater => tex::LAMP_RED,
        Status::NoDeposit | Status::Exhausted | Status::Overheated => tex::LAMP_RED,
        Status::NoInput => tex::FRAME,
    }
}
