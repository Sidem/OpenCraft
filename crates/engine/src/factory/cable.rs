//! How cables look (their rules are `pole.rs`): a copper knot in the cell with a thin arm towards every
//! neighbouring cell that holds a cable, pole or machine, so a cable hung down a shaft reads as one
//! line. A cable with nothing beside it draws a short vertical stub. Presentation only.
//!
//! `write_cables` draws the placed ones (called with the other machines: `render.rs`); `preview_cable`
//! draws one for the placement ghost (`power_tools.rs`).

use crate::block::tex;
use crate::math::{IVec3, Vec3};

use super::render::push_box;
use super::Factory;

/// The six neighbours: +x, -x, +y, -y, +z, -z.
const AROUND: [(i32, i32, i32); 6] = [(1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)];
const COPPER: [u16; 3] = [tex::COPPER_WIRE; 3];
const THIN: f32 = 0.07;

/// One cable at camera-relative `rel` (its cell's centre) with an arm towards each neighbour in `arms`.
pub fn preview_cable(out: &mut Vec<f32>, rel: Vec3, arms: [bool; 6]) {
    push_box(out, rel, 0.0, [0.16, 0.16, 0.16], 0.0, COPPER, false);
    let stub = !arms.iter().any(|&a| a);
    for (i, &(x, y, z)) in AROUND.iter().enumerate() {
        if arms[i] || stub && y != 0 {
            let at = rel + Vec3::new(x as f64, y as f64, z as f64) * 0.25;
            let size =
                [if x != 0 { 0.5 } else { THIN }, if y != 0 { 0.5 } else { THIN }, if z != 0 { 0.5 } else { THIN }];
            push_box(out, at, 0.0, size, 0.0, COPPER, false);
        }
    }
}

impl Factory {
    /// Boxes for every cable within `range` of the camera.
    pub(super) fn write_cables(&self, out: &mut Vec<f32>, eye: Vec3, range: f64) {
        for c in self.poles.iter().filter(|p| p.is_cable()) {
            let rel = c.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
            if rel.x * rel.x + rel.y * rel.y + rel.z * rel.z > range * range {
                continue;
            }
            let arms = AROUND.map(|(x, y, z)| self.at.contains_key(&(c.pos + IVec3::new(x, y, z))));
            preview_cable(out, rel, arms);
        }
    }
}
