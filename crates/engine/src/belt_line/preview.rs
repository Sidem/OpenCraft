//! What a dragged belt line shows before it is built (part of `belt_line.rs`): ghost belts, underpasses and lift
//! columns along the planned cells ([`Game::write_line_preview`]), and the host's outline boxes with whether each
//! cell is paid for ([`Game::planned_cells`]). Presentation only: reads the plan and the inventory, changes nothing.

use crate::factory::{self, tiers, upgrades};
use crate::math::{IVec3, Vec3};
use crate::Game;

use super::{pass, Piece, UP};

impl Game {
    /// Ghost belts along the planned line, drawn with the machines; a lift column is drawn as one.
    pub(crate) fn write_line_preview(&mut self, eye: Vec3, time: f64) {
        let held = self.inventory().selected_stack().item;
        if upgrades::kit_tier(held).is_some() {
            return; // upgrades show as coloured outlines only; the belts are already there
        }
        let tier = tiers::placed_by(held).map_or(0, |(_, t)| t);
        let cells = &self.line.cells;
        let lift_at = |p: IVec3| cells.iter().any(|c| c.piece == Piece::Lift && c.pos == p);
        for c in cells {
            let rel = c.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5) - eye;
            if c.piece == Piece::Lift {
                let stack = (lift_at(c.pos - UP), lift_at(c.pos + UP));
                factory::lift_preview(&mut self.instances, c.pos, c.dir, c.shape, stack, rel, time);
                continue;
            }
            let tier = if c.piece == Piece::Belt { tier } else { c.tier };
            factory::belt_preview(&mut self.instances, c.pos, c.dir, c.shape, tier, rel, time);
        }
    }

    /// Cells of the planned line for the host's outlines: x, y, z and 1 if it will be built, or the kit's
    /// tier colour (0xRRGGBB) if it will be upgraded; 0 (red) past what the held stack pays for, for
    /// underpasses the inventory has no pair of, and for what is too wide to go under.
    pub(crate) fn planned_cells(&self) -> Vec<i32> {
        let have = self.line_budget();
        let kit = upgrades::kit_tier(self.inventory().selected_stack().item);
        let colour = kit.map_or(1, |t| upgrades::TIER_COLOURS[t as usize] as i32);
        let paid: Vec<bool> = match kit {
            Some(_) => (0..self.line.cells.len()).map(|i| i < have).collect(),
            None => pass::affordable(&self.line.cells, have),
        };
        let cell = |p: IVec3, ok: bool| [p.x, p.y, p.z, if ok { colour } else { 0 }];
        let planned = self.line.cells.iter().zip(paid).map(|(c, ok)| cell(c.pos, ok));
        let aimed = self.line.aim.belts.iter().enumerate().map(|(i, &p)| cell(p, self.line.cells.len() + i < have));
        let blocked = self.line.blocked.map(|p| cell(p, false));
        planned.chain(aimed).chain(blocked).flatten().collect()
    }
}
