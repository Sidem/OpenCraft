//! Processor models as data: a spec's `parts` are boxes (centre and size in block units, relative to
//! the footprint's centre, front towards +z) turned with the machine, each with a `Look`. The tier band (with the
//! Mk studs of `tier_mark.rs` round it), the product display (`display.rs`), the status lamp, a fire that glows while working, a press that pumps and a chimney's smoke are looks, so a new
//! processor is rows, not drawing code. Round bodies are real ones (`round`, `round_x`: `push_round`). A multi-block
//! machine's port hatches are drawn from its footprint.

use crate::block::tex;
use crate::math::Vec3;
use crate::TICK;

use super::super::footprint::Role;
use super::super::render::{push_box, push_shape, Axis, Form, UPRIGHT};
use super::super::{smoke, ticks, tier_mark, DIRS};
use super::{steam_view, Energy, Processor, Status};

#[cfg(test)]
mod tests;

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
    /// The mouth of a chimney or exhaust: no box, but smoke while working (`factory/smoke.rs`), puffs scaled by
    /// the part's width.
    Smoke,
    /// Where the rocket stands (`rocket.rs`, the part's centre on the mount): drawn only while a batch runs, its engines
    /// lighting at `IGNITE_AT` and lifting it off at `LIFT_AT`.
    Rocket,
    /// A solid mark in the machine's channel colour (a pod launcher or pad: `pods/channels.rs`).
    Channel,
    /// The product display (`display.rs`): the part is its slot (square, on a roof), a turning plate at its floor and
    /// the product standing on it. Every machine that makes items has one (tested).
    Product,
}

#[derive(Clone, Copy)]
pub struct Part {
    pub at: [f32; 3],
    pub size: [f32; 3],
    pub look: Look,
    /// Extra turn about its own centre, in quarter turns (a mirror's plate stands at 45°).
    pub turn: f32,
    /// A box, or a round body filling the box (`push_round`).
    pub form: Form,
}

pub const fn part(at: [f32; 3], size: [f32; 3], look: Look) -> Part {
    Part { at, size, look, turn: 0.0, form: Form::Box }
}

/// A part turned `turn` quarter turns more than its machine.
pub const fn turned(at: [f32; 3], size: [f32; 3], look: Look, turn: f32) -> Part {
    Part { at, size, look, turn, form: Form::Box }
}

/// An upright round body (a tank, column or drum) `size` across, high and deep.
pub const fn round(at: [f32; 3], size: [f32; 3], look: Look) -> Part {
    Part { at, size, look, turn: 0.0, form: UPRIGHT }
}

/// A round body lying along x (a boiler drum), `size` long, high and deep.
pub const fn round_x(at: [f32; 3], size: [f32; 3], look: Look) -> Part {
    Part { at, size, look, turn: 0.0, form: Form::Round(Axis::X, 1.0) }
}

/// Deepest press stroke, in blocks.
const STROKE: f64 = 0.12;
/// A rocket lights its engines when its batch is this far along (thousandths), starts to rise at the next and climbs this
/// many blocks by the end, accelerating all the way.
const IGNITE_AT: f64 = 895.0;
const LIFT_AT: f64 = 905.0;
const LIFT_HEIGHT: f64 = 160.0;
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
            Look::Smoke => {
                if working {
                    smoke::puffs(out, at, time, smoke::seed(p.pos), part.size[0]);
                }
                continue;
            }
            Look::Product => {
                super::display::draw(p, out, at, yaw, part.size, time);
                continue;
            }
            Look::Tex(t) => t,
            Look::Band(top) => {
                // A machine that comes in tiers wears its Mk studs around its band.
                if p.spec.tiers.len() > 1 {
                    tier_mark::ring(out, at, yaw, [part.size[0] / 2.0, part.size[2] / 2.0], p.tier);
                }
                [top, tex::stripe(p.tier), tex::FRAME]
            }
            Look::Fire(idle) => [if working { tex::LAMP_YELLOW } else { idle }; 3],
            Look::Lamp => [lamp(p.status); 3],
            Look::Press(share, t) => {
                at.y -= stroke * share as f64;
                t
            }
            Look::Channel => [tex::GRID_FIRST + u16::from(p.channel()); 3],
            Look::Rocket => {
                if let Some(r) = p.batch_recipe().filter(|_| p.batch.is_some()) {
                    // Blended into the tick like the drones, at last tick's speed, so the climb is smooth.
                    let rate = f64::from(p.stats().speed) * f64::from(p.speed) / 1e3 * f64::from(p.boost) / 1e3;
                    let ahead = if working { (time / TICK).fract() * rate } else { 0.0 };
                    let done = (f64::from(p.progress) + ahead) / f64::from(ticks(r.seconds));
                    let lift = ((done - LIFT_AT) / (1000.0 - LIFT_AT)).clamp(0.0, 1.0);
                    let thrust = ((done - IGNITE_AT) / (LIFT_AT - IGNITE_AT)).clamp(0.0, 1.0) as f32;
                    let base = at - Vec3::new(0.0, f64::from(part.size[1]) / 2.0, 0.0);
                    super::rocket::draw(out, base, yaw, lift * lift * LIFT_HEIGHT, thrust, time);
                }
                continue;
            }
        };
        push_shape(out, part.form, at, yaw + part.turn * std::f32::consts::FRAC_PI_2, part.size, texture, false);
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
