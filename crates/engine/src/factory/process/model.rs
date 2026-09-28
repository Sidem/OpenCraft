//! Processor models as data: a spec's `parts` are boxes in its cell (centre and size in block units,
//! relative to the cell centre), each with a `Look`. The tier band, the status lamp, a fire that glows
//! while working and a press that pumps are looks, so a new processor is rows, not drawing code.

use crate::block::tex;
use crate::math::Vec3;

use super::super::render::push_box;
use super::{Processor, Status};

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

/// Draws `p`'s parts at `rel` (its cell centre relative to the camera).
pub fn draw(p: &Processor, out: &mut Vec<f32>, rel: Vec3, time: f64) {
    let working = p.status == Status::Working;
    let stroke = if working { (time * 6.0).sin().abs() * STROKE } else { 0.0 };
    for part in p.spec.parts {
        let mut at = rel + Vec3::new(part.at[0] as f64, part.at[1] as f64, part.at[2] as f64);
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
        push_box(out, at, 0.0, part.size, 0.0, texture, false);
    }
}

fn lamp(status: Status) -> u16 {
    match status {
        Status::Working => tex::LAMP_GREEN,
        Status::OutputFull => tex::LAMP_YELLOW,
        Status::NoRecipe | Status::NoPower | Status::NoFuel => tex::LAMP_RED,
        Status::NoInput => tex::FRAME,
    }
}
