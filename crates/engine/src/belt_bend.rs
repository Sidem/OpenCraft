//! Bending belts by hand (Shift+R): a belt that takes items in from its left or right instead of its back, so a
//! machine can feed a belt that runs along it, or a belt can turn right beside a machine (`factory/belt_curve.rs`).
//! Looking at a flat belt, Shift+R steps its bend (an action, so co-op and saves see it); otherwise, with belts in
//! hand, it steps the bend the first belt of the next line gets (`held_bend`; the line preview shows it). Part of the
//! local player's hands, not core.

use crate::action::Action;
use crate::belt_line::{is_belt, Piece};
use crate::block;
use crate::factory::{self, Shape, BENDS};
use crate::math::Vec3;
use crate::sound;
use crate::Game;

impl Game {
    /// Shift+R: steps the bend of the targeted flat belt, or else the bend the next line's first belt gets. False when
    /// neither applies.
    pub(crate) fn bend_target_belt(&mut self) -> bool {
        let aimed = self.target.and_then(|hit| Some((hit.block, self.sim.factory.bend_at(hit.block)?)));
        if let Some((pos, bend)) = aimed {
            self.act(Action::BendBelt { pos, bend: (bend + 1) % BENDS });
            self.play(sound::PLACE, block::sound::METAL, pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5), 0.6);
            return true;
        }
        if !self.inventory().selected_stack().item.places().is_some_and(is_belt) {
            return false;
        }
        self.held_bend = (self.held_bend + 1) % BENDS;
        self.play(sound::PLACE, block::sound::METAL, self.body().eye(), 0.4);
        true
    }

    /// The bend the next line's first belt gets (`factory::BEND_STRAIGHT`, `BEND_LEFT` or `BEND_RIGHT`).
    pub(crate) fn next_bend(&self) -> u8 {
        self.held_bend
    }

    /// What the HUD says with belts in hand and a bend held (`api/hud.rs` `line_label`), before a line is dragged.
    pub(crate) fn bend_label(&self) -> Option<String> {
        let belts = self.inventory().selected_stack().item.places().is_some_and(is_belt);
        let side = match self.held_bend {
            factory::BEND_LEFT => "left",
            factory::BEND_RIGHT => "right",
            _ => return None,
        };
        belts.then(|| format!("Bent belt\nThe next belt you lay takes items in from its {side} · Shift+R to change"))
    }

    /// Puts the held bend on the planned line's first cell if it is a flat belt: its `dir` then carries the bend in its
    /// upper bits (`factory::belt_facing`), which the preview, the placements and the ghosts pass on. Call it last,
    /// once nothing else reads the cells' directions.
    pub(crate) fn bend_line_start(&mut self) {
        let bend = self.held_bend;
        let first = self.line.cells.first_mut().filter(|c| c.piece == Piece::Belt && c.shape == Shape::Flat);
        if let Some(c) = first {
            c.dir = factory::belt_facing(c.dir, bend);
        }
    }
}

#[cfg(test)]
mod tests;
