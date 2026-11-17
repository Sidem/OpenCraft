//! A multi-block machine's ports as openings (presentation): at each port face (its footprint's `In`, `Out` and `Side`
//! rows) a doorway framed by two jambs in the machine's tier stripe under a lintel in the port's hatch colour
//! (`tex::PORT_*`: blue in, orange out, purple byproduct), filled with darkness: a face that takes no light, which items
//! on a belt vanish into (an item is handed over at the cell border, its front half already past the dark) and come out
//! of. Machine walls stand 2 cm inside their cells, so the dark is a face just in front of the wall, not a hollow.
//! - A port with a belt joined (`Processor::linked`, from `relink`) lights a lamp on the lintel's top in the port's
//!   colour; a port with no belt, or one that doesn't work there, stays dark.
//! - Beyond `DETAIL_RANGE` only a plain dark mouth and the lintel are drawn (darkness ignores the fog, so far off it would
//!   show through it).
//!
//! Sizes keep every face clear of its neighbours' (`tests/face_fight.rs`). To change the look: the constants here; which
//! faces are ports lives in the footprint (`footprint/`).

use std::f32::consts::FRAC_PI_2;

use crate::block::tex;
use crate::math::Vec3;

use super::super::belt::ITEM_DETAIL_RANGE;
use super::super::footprint::Role;
use super::super::port_marks::{arrow_look, Flow};
use super::super::render::{mark_beacon_tinted, mark_dark, push_box};
use super::super::DIRS;
use super::Processor;

/// Beyond this many blocks a port is its mouth and lintel only.
const DETAIL_RANGE: f64 = ITEM_DETAIL_RANGE + 2.0;
/// Depths are out from the cell's centre (its border at 0.5: a machine's model stays inside its cells), heights up from
/// its floor. The dark: across (the doorway between the jambs), high, deep; how far out its face stands (in front of
/// machine walls, which stand at 0.48 or less behind a port).
const DARK: [f32; 3] = [0.62, 0.5, 0.02];
const DARK_FLOOR: f32 = 0.04;
const DARK_FACE: f32 = 0.494;
/// A jamb: across, high (from just over the floor, so it shares no face with a floor plate), deep; its centre's
/// distance from the side's middle. It stops under the lintel.
const JAMB: [f32; 3] = [0.08, 0.53, 0.1];
const JAMB_FLOOR: f32 = 0.01;
const JAMB_X: f32 = 0.35;
/// The lintel over the doorway: across (off the round numbers machine walls stand at), high, deep.
const LINTEL: [f32; 3] = [0.776, 0.1, 0.1];
/// Frame parts' centres stand this far out from the cell's centre.
const FRAME_OUT: f32 = 0.45;
/// The lamp on the lintel's top that lights when a belt is joined: across, high, deep.
const LAMP: [f32; 3] = [0.6, 0.012, 0.05];

/// Draws `p`'s port openings; `rel` is its anchor cell's centre relative to the camera.
pub(in crate::factory) fn draw(p: &Processor, out: &mut Vec<f32>, rel: Vec3) {
    let near = rel.length() <= DETAIL_RANGE;
    let stripe = tex::stripe(p.tier_colour());
    for (k, (role, cell, side)) in p.port_faces().into_iter().enumerate() {
        let (layer, flow) = match role {
            Role::In => (tex::PORT_IN, Flow::In),
            Role::Out => (tex::PORT_OUT, Flow::Out),
            _ => (tex::PORT_SIDE, Flow::Byproduct),
        };
        let port = Port { centre: rel + (cell - p.pos).as_vec3(), side };
        port.mouth(out, layer, near);
        if near {
            port.frame(out, stripe);
            if k < 64 && p.linked & 1 << k != 0 {
                port.lamp(out, arrow_look(flow));
            }
        }
    }
}

/// One port face: its cell's camera-relative centre and the `DIRS` index it looks out of.
struct Port {
    centre: Vec3,
    side: u8,
}

impl Port {
    /// A point `out` from the cell's centre through the face, `x` across it (to the right looking out) and `y` up
    /// from the cell's floor.
    fn at(&self, x: f32, y: f32, out: f32) -> Vec3 {
        let away = DIRS[self.side as usize].as_vec3();
        let across = Vec3::new(-away.z, 0.0, away.x);
        self.centre + away * f64::from(out) + across * f64::from(x) + Vec3::new(0.0, f64::from(y - 0.5), 0.0)
    }

    /// The yaw that turns a box's own +z out of the face (a box's +z looks along (-sin yaw, 0, cos yaw)).
    fn yaw(&self) -> f32 {
        f32::from((self.side + 2) % 4) * FRAC_PI_2
    }

    /// The dark in the doorway (near; far off a plain dark box) and the lintel over it, in the port's hatch colour.
    fn mouth(&self, out: &mut Vec<f32>, layer: u16, near: bool) {
        let dark = self.at(0.0, DARK_FLOOR + DARK[1] / 2.0, DARK_FACE - DARK[2] / 2.0);
        push_box(out, dark, self.yaw(), DARK, 0.0, [tex::BLACK_CONCRETE; 3], false);
        if near {
            mark_dark(out);
        }
        let lintel = self.at(0.0, DARK_FLOOR + DARK[1] + LINTEL[1] / 2.0, FRAME_OUT);
        push_box(out, lintel, self.yaw(), LINTEL, 0.0, [tex::FRAME, layer, tex::FRAME], false);
    }

    /// The two jambs, in the machine's tier stripe.
    fn frame(&self, out: &mut Vec<f32>, stripe: u16) {
        for x in [-JAMB_X, JAMB_X] {
            let at = self.at(x, JAMB_FLOOR + JAMB[1] / 2.0, FRAME_OUT);
            push_box(out, at, self.yaw(), JAMB, 0.0, [tex::FRAME, stripe, tex::FRAME], false);
        }
    }

    /// The lamp on the lintel's top, lit in `colour`: a belt is joined.
    fn lamp(&self, out: &mut Vec<f32>, colour: u32) {
        let top = DARK_FLOOR + DARK[1] + LINTEL[1];
        let at = self.at(0.0, top + LAMP[1] / 2.0 + 0.003, FRAME_OUT);
        push_box(out, at, self.yaw(), LAMP, 0.0, [tex::FRAME; 3], false);
        mark_beacon_tinted(out, colour);
    }
}
