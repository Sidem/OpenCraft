//! Rails (Milestone 9): track a train will run on. A rail joins the rails beside it, on its own level and one
//! block up or down (a track climbs and drops a block a cell, like belt ramps), and has no direction of its own:
//! a cell of track is a node, its joins are the edges, and the graph is derived (`link_rails`, in `relink`),
//! never saved. Stateless: a rail only remembers where it is.
//!
//! Invariants: joining is symmetric (a rail rising towards `q` is the same join as `q` dropping towards it, see
//! [`joins`]); `arms` is derived data. The sloped piece of a join lives in its lower cell, like a belt ramp.
//!
//! To use the track: `joins` and `Factory::rail_joins` give a cell's neighbours (trains, Milestone 9 step 9.4b).

use crate::block::tex;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::math::{IVec3, Vec3};

use super::belt_shape::Shape;
use super::links::Slot;
use super::render::push_box;
use super::{Factory, Machine, DIRS};

/// How a join leaves a rail: level, rising a block, or dropping a block.
const LEVEL: u8 = 0;
const RISE: u8 = 1;
const DROP: u8 = 2;
const HEIGHT_CHANGES: [i32; 3] = [0, 1, -1];

/// Rail geometry (block units, from the cell centre): the rails' height, their gap and thickness, a sleeper's size.
const RAIL_Y: f32 = -0.38;
const GAUGE: f32 = 0.18;
const RAIL_THICK: f32 = 0.06;
const SLEEPER_Y: f32 = -0.45;

pub struct Rail {
    pub pos: IVec3,
    /// Joins to other rails, bit `dir * 3 + kind` (`LEVEL`, `RISE`, `DROP`; derived).
    pub arms: u16,
}

impl Rail {
    pub fn new(pos: IVec3) -> Rail {
        Rail { pos, arms: 0 }
    }

    /// How many rails this one joins.
    pub fn joined(&self) -> u32 {
        self.arms.count_ones()
    }
}

/// The twelve cells a rail at `p` can join, as (direction, kind, cell): each side level, up and down.
pub fn joins(p: IVec3) -> impl Iterator<Item = (u8, u8, IVec3)> {
    (0..4u8).flat_map(move |d| {
        (0..3u8).map(move |k| (d, k, p + DIRS[d as usize] + IVec3::new(0, HEIGHT_CHANGES[k as usize], 0)))
    })
}

impl Factory {
    /// Records each rail's joins (called from `relink`).
    pub(super) fn link_rails(&mut self) {
        let at = &self.at;
        for r in &mut self.rails {
            r.arms = joins(r.pos)
                .filter(|&(_, _, q)| matches!(at.get(&q), Some(Slot::Rail(_))))
                .fold(0, |a, (d, k, _)| a | 1 << (d * 3 + k));
        }
    }

    /// The rails the rail at `pos` joins (none for any other cell). Tests for now: trains (9.4b) route along it.
    #[cfg(test)]
    pub fn rail_joins(&self, pos: IVec3) -> Vec<IVec3> {
        let Some(Slot::Rail(_)) = self.at.get(&pos) else { return Vec::new() };
        joins(pos).filter(|(_, _, q)| matches!(self.at.get(q), Some(Slot::Rail(_)))).map(|j| j.2).collect()
    }
}

impl Machine for Rail {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
    }

    fn read_state(r: &mut ByteReader) -> Option<Rail> {
        Some(Rail::new(r.ivec3()?))
    }

    fn contents(&self) -> Vec<Stack> {
        Vec::new()
    }

    fn describe(&self, f: &Factory) -> String {
        if f.dirty {
            return String::new();
        }
        match self.joined() {
            0 => "Rail\nJoins the rails laid beside it, level or a block up or down".to_string(),
            1 => "Rail\nEnd of the track: joined to 1 rail".to_string(),
            n => format!("Rail\nJoined to {n} rails"),
        }
    }

    fn model(&self, out: &mut Vec<f32>, rel: Vec3, _: f64) {
        write_rail(out, rel, self.arms);
    }
}

/// The joins a planned rail (the line tool's ghost) will have: straight along `dir`, sloping when `shape` says its
/// neighbour is a block up (`Up`) or it is the foot of a rise from the cell behind (`Down`).
pub fn ghost_arms(dir: u8, shape: Shape) -> u16 {
    let (ahead, behind) = (dir % 4, (dir + 2) % 4);
    let bit = |d: u8, k: u8| 1u16 << (d * 3 + k);
    match shape {
        Shape::Up => bit(ahead, RISE) | bit(behind, LEVEL),
        Shape::Down => bit(ahead, LEVEL) | bit(behind, RISE),
        _ => bit(ahead, LEVEL) | bit(behind, LEVEL),
    }
}

/// A rail's boxes at camera-relative `rel` (its cell's centre) with the joins in `arms`: a sleeper under the
/// middle and a pair of steel rails towards each join (a bare rail runs north-south).
pub fn write_rail(out: &mut Vec<f32>, rel: Vec3, arms: u16) {
    let arms = if arms == 0 { 1 << (LEVEL as u16) | 1 << (2 * 3 + LEVEL as u16) } else { arms };
    push_box(out, rel + Vec3::new(0.0, SLEEPER_Y as f64, 0.0), 0.0, [0.5, 0.05, 0.14], 0.0, [tex::PLANKS; 3], false);
    for d in 0..4u8 {
        for k in [LEVEL, DROP, RISE] {
            // A drop is drawn by the lower cell's rise, so its half stays level here.
            if arms & (1 << (d * 3 + k)) != 0 {
                arm(out, rel, d, k == RISE);
            }
        }
    }
}

/// One half-cell of track towards direction `d`, tilted up when it `rises`.
fn arm(out: &mut Vec<f32>, rel: Vec3, d: u8, rises: bool) {
    let yaw = d as f32 * std::f32::consts::FRAC_PI_2;
    let (s, c) = yaw.sin_cos();
    // Local -z points along `DIRS[d]`; (x, y, z) are local offsets.
    let at = |x: f32, y: f32, z: f32| rel + Vec3::new((c * x - s * z) as f64, y as f64, (s * x + c * z) as f64);
    let (len, lift, pitch) = if rises { (0.71, 0.25, std::f32::consts::FRAC_PI_4) } else { (0.5, 0.0, 0.0) };
    let mut rail = |x: f32, y: f32, z: f32, size: [f32; 3], layer: u16| {
        push_box(out, at(x, y, z), yaw, size, 0.0, [layer; 3], false);
        let n = out.len();
        out[n - 4] = pitch;
    };
    for x in [-GAUGE, GAUGE] {
        rail(x, RAIL_Y + lift, -0.25, [RAIL_THICK, 0.08, len], tex::STEEL);
    }
    rail(0.0, SLEEPER_Y + lift, -0.3, [0.5, 0.05, 0.14], tex::PLANKS);
}

#[cfg(test)]
mod tests;
