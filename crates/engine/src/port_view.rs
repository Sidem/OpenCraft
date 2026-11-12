//! How a multi-block machine's ports read (the local player's, presentation only): while one is held and aimed, a
//! see-through ghost of it stands where it would go (red when something is in the way) with its port marks
//! (`factory/port_marks.rs`: a belt stub, arrows and the item carried at every face that takes or gives items); while a
//! belt is held, built machines nearby show the same marks. The HUD names the sides as the player sees them
//! (`ports_line`). Every answer comes from the footprint's port rows, the data the simulation uses, so what is shown
//! cannot disagree with what works (`port_view/tests.rs` belts every marked face of every machine at every turn).
//! Nothing here touches the core.

use crate::block::{self, BlockId};
use crate::factory::port_marks::{self, Flow};
use crate::factory::{self, footprint, Kind};
use crate::math::{IVec3, Vec3};
use crate::Game;

/// This frame's see-through box instances (`factory::INSTANCE_FLOATS` each), the port marks (solid, drawn after the
/// ghost so they show through it, though not through the world), each with its round bodies copied out as for the
/// world's boxes, and whether the ghost is in the way of something (the host tints it red).
#[derive(Default)]
pub(crate) struct Ghosts {
    pub boxes: Vec<f32>,
    pub round: Vec<f32>,
    pub marks: Vec<f32>,
    pub marks_round: Vec<f32>,
    pub blocked: bool,
}

/// Every machine block of more than one cell that turns (a long-distance pole does not).
pub(crate) fn footprint_blocks() -> Vec<u8> {
    (1..=255u8).filter(|&b| b != block::LONG_POLE && footprint::of(b).is_some()).collect()
}

/// Which side a face looking out along `DIRS[s]` is to a player facing `DIRS[facing]`: 0 far, 1 right, 2 near, 3 left.
pub(crate) fn seen_side(s: u8, facing: u8) -> u8 {
    (s + 4 - facing % 4) % 4
}

impl Game {
    /// Draws the held machine's ghost and its port marks, or the marks of built machines while a belt is held.
    pub(crate) fn write_port_view(&mut self, eye: Vec3, time: f64) {
        let ghost = self.footprint_ghost();
        let belt = self.holds_belt();
        let world = &self.sim.world;
        let free = |c: IVec3| world.get_block(c).is_some_and(block::replaceable);
        let Ghosts { boxes, round, marks, marks_round, blocked } = &mut self.ghosts;
        boxes.clear();
        marks.clear();
        *blocked = false;
        if let Some((block, anchor, dir, cells)) = ghost {
            *blocked = cells.iter().any(|c| c.1);
            port_marks::draw_ghost(block, anchor, dir, eye, time, free, marks, boxes);
        } else if belt {
            self.sim.factory.write_port_marks(eye, time, free, marks, boxes);
        }
        factory::split_round(boxes, round);
        factory::split_round(marks, marks_round);
    }

    /// The ports of `block` placed `turn` quarter turns from the way the player faces, as the player sees them: pairs
    /// of (side: 0 far, 1 right, 2 near, 3 left; flow: 0 in, 1 out, 2 byproduct), one per face.
    pub(crate) fn ports_as_seen(&self, block: BlockId, turn: u8) -> Vec<u8> {
        let facing = factory::dir_from_yaw(self.body().yaw);
        let dir = footprint::layout_dir(block, facing + turn);
        let mut out = Vec::new();
        for (_, s, flow) in port_marks::port_faces(block, IVec3::ZERO, dir) {
            out.extend([seen_side(s, facing), flow as u8]);
        }
        out
    }

    /// "in: left, far, right · out: near · byproduct: right" for the held machine at its current turn ("" with none).
    pub(crate) fn ports_line(&self, block: BlockId) -> String {
        const SIDES: [&str; 4] = ["far", "right", "near", "left"];
        let seen = self.ports_as_seen(block, self.place_turn);
        let mut parts = Vec::new();
        for (flow, word) in [(Flow::In, "in"), (Flow::Out, "out"), (Flow::Byproduct, "byproduct")] {
            // Each side once, in the order far, right, near, left.
            let sides: Vec<&str> = (0..4u8)
                .filter(|&s| seen.chunks_exact(2).any(|p| p[0] == s && p[1] == flow as u8))
                .map(|s| SIDES[s as usize])
                .collect();
            if !sides.is_empty() {
                parts.push(format!("{word}: {}", sides.join(", ")));
            }
        }
        parts.join(" · ")
    }

    fn holds_belt(&self) -> bool {
        let stack = self.inventory().selected_stack();
        !stack.is_empty() && stack.item.places().and_then(factory::machine).is_some_and(|m| m.kind == Kind::Belt)
    }
}

#[cfg(test)]
mod tests;
