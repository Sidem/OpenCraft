//! Street lamps (`block::STREET` and its post cells): a post four cells tall with a bent arm and a lantern at the top. It
//! has no state of its own, only blocks: the head, `STREET`, on top (the item; light row `LED_LIGHTS[2]`, the brightest), and
//! three `street_post(dir)` cells under it, `dir` being the way the arm leans (the way the placer faced, so it reaches
//! over what the placer was looking at). The model is drawn from the head (`view.rs`, `model.rs`).
//!
//! Invariants: placing needs all four cells free (air or water: `block::replaceable`) and sets them together; breaking any
//! cell clears the other three (`street_cell_changed`, from `Sim::block_changed`) and drops one item, from the cell broken.
//! A lamp is the head and the three cells below it, so lamps stacked on each other stay apart (a head ends a column).
//! Nothing is saved, hashed or sent beyond the blocks, so the save format and the state hash are unchanged but for the new
//! ids.
//!
//! To change the look: `model.rs` (boxes inside the 1×1×4 column) and the lantern's place, [`LANTERN_OUT`] / [`LANTERN_Y`],
//! which the halo (`halos.rs`) shares.

mod model;
mod view;

use crate::block::{self, replaceable, AIR, STREET};
use crate::factory::DIRS;
use crate::math::{IVec3, Vec3};
use crate::sim::{PlayerId, Sim, SimEvent};
use crate::world::World;

pub use model::street_model;

/// Cells a lamp stands in, post first and head last.
pub const HEIGHT: i32 = 4;
/// Where the lantern's middle hangs: how far out toward the arm's way from the column's middle, and how high above the
/// column's middle (the column runs from -2 to 2; the head cell's middle is 1.5). The model and the halo both use it.
pub const LANTERN_OUT: f64 = 0.3;
pub const LANTERN_Y: f64 = 1.45;

const UP: IVec3 = IVec3::new(0, 1, 0);

/// The way the arm of the lamp whose head is at `head` leans (a `DIRS` index), read from the post below it; 0 until
/// that cell is loaded.
pub fn arm_dir(world: &World, head: IVec3) -> u8 {
    world.get_block(head - UP).and_then(block::post_dir).unwrap_or(0)
}

/// Where the lantern's middle is in the world, for the lamp whose head is at `head`.
pub fn lantern_at(head: IVec3, dir: u8) -> Vec3 {
    head.as_vec3() + Vec3::new(0.5, 0.5 + LANTERN_Y - 1.5, 0.5) + DIRS[dir as usize % 4].as_vec3() * LANTERN_OUT
}

impl Sim {
    /// Puts a street lamp with its base cell at `pos`, its arm leaning the way a player facing `facing` looks, if the four
    /// cells are free (`put_block`).
    pub(crate) fn put_street_lamp(&mut self, player: PlayerId, pos: IVec3, facing: u8) -> bool {
        let dir = facing % 4;
        let cells: Vec<IVec3> = (0..HEIGHT).map(|k| pos + UP * k).collect();
        let olds: Vec<_> = cells.iter().map(|&c| self.world.block_anywhere_or_generate(c)).collect();
        if !olds.iter().all(|&b| replaceable(b)) {
            return false;
        }
        for (k, &c) in cells.iter().enumerate() {
            let b = if k as i32 == HEIGHT - 1 { STREET } else { block::street_post(dir) };
            if !self.world.set_block_anywhere(c, b) {
                for (&c, &old) in cells.iter().zip(&olds).take(k) {
                    self.world.set_block_anywhere(c, old);
                }
                return false;
            }
        }
        self.events.push(SimEvent::BlockPlaced { player, pos: cells[HEIGHT as usize - 1], block: STREET });
        for (c, old) in cells.into_iter().zip(olds) {
            self.block_changed(c, old);
        }
        true
    }

    /// After the block at `pos` changed from `old`: a lamp that lost a cell is gone, every other cell of it too.
    pub(crate) fn street_cell_changed(&mut self, pos: IVec3, old: block::BlockId) {
        if !block::is_street(old) || block::is_street(self.block(pos)) {
            return;
        }
        // The head is the broken cell, or the first head above a post (the three cells above a post at most).
        let mut head = pos;
        if old != STREET {
            let mut at = pos;
            loop {
                at = at + UP;
                match self.block(at) {
                    STREET => {
                        head = at;
                        break;
                    }
                    b if block::post_dir(b).is_some() && at.y - pos.y < HEIGHT => {}
                    _ => return,
                }
            }
        }
        for k in 0..HEIGHT {
            let c = head - UP * k;
            let b = self.block(c);
            let part = if k == 0 { b == STREET } else { block::post_dir(b).is_some() };
            if c != pos && part && self.world.set_block_anywhere(c, AIR) {
                self.block_changed(c, b);
            }
        }
    }
}

#[cfg(test)]
mod tests;
