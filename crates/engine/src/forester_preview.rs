//! Placing a forester (the local player's, presentation only): while one is held and aimed at a free cell, the patch
//! it would work is outlined (green; red with a red box on every uneven cell when its ground is not flat enough to place
//! it), the label says how big it is and what stands on it, and a click on uneven ground is refused with a "no". R turns
//! it (`quarry_preview.rs`). Reads loaded chunks only; the core checks the ground again (`action/blocks.rs`).

use crate::block::{self, is_log, AIR, FORESTER};
use crate::factory::{forester_survey, tiers, ForesterPatch, ForesterSurvey};
use crate::math::IVec3;
use crate::upgrade_aim::BLOCKED_RED;
use crate::Game;

/// The patch outline's colour when it can be placed.
const PATCH_GREEN: i32 = 0x6fbf4a;
/// Uneven cells marked at most.
const MARKS: usize = 64;

/// A held forester aimed at a free cell: where it would stand, which way it would face, its tier, and its ground.
struct Plan {
    pos: IVec3,
    dir: u8,
    tier: u8,
    ground: ForesterSurvey,
}

impl Game {
    fn forester_plan(&self) -> Option<Plan> {
        let hit = self.target.filter(|h| h.normal != IVec3::ZERO)?;
        let stack = self.inventory().selected_stack();
        if stack.is_empty() || stack.item.places() != Some(FORESTER) {
            return None;
        }
        let pos = hit.block + hit.normal;
        self.sim.world.get_block(pos).filter(|&b| block::replaceable(b))?;
        let (dir, tier) = (self.placing_facing(), tiers::placed_by(stack.item).map_or(0, |(_, t)| t));
        let world = &self.sim.world;
        let ground = forester_survey(pos, dir, tier, |p| world.get_block(p).unwrap_or(AIR));
        Some(Plan { pos, dir, tier, ground })
    }

    /// Where a click would be refused because the ground in front is not flat enough.
    pub(crate) fn forester_refused(&self) -> Option<IVec3> {
        self.forester_plan().filter(|p| p.ground.size.is_none()).map(|p| p.pos)
    }

    /// The patch outline and the uneven cells' boxes (the placing outline's format: `Game::placement_box`).
    pub(crate) fn forester_boxes(&self) -> Vec<i32> {
        let Some(plan) = self.forester_plan() else { return Vec::new() };
        let patch = ForesterPatch { pos: plan.pos, dir: plan.dir, size: plan.ground.size.unwrap_or(0) };
        let (lo, hi) = patch.bounds();
        let colour = if plan.ground.size.is_some() { PATCH_GREEN } else { BLOCKED_RED };
        let mut boxes = vec![lo.x, lo.y, lo.z, hi.x, lo.y, hi.z, colour];
        if plan.ground.size.is_none() {
            let world = &self.sim.world;
            for c in patch.uneven(|p| world.get_block(p).unwrap_or(AIR)).into_iter().take(MARKS) {
                boxes.extend([c.x, c.y - 1, c.z, c.x, c.y, c.z, BLOCKED_RED]);
            }
        }
        boxes
    }

    /// "Forester Mk1" and "11×11 patch, 9 plots · 3 trees to fell · R turns it" on two lines, or why it can't go here
    /// ("" when not placing one).
    pub(crate) fn forester_label(&self) -> String {
        let Some(plan) = self.forester_plan() else { return String::new() };
        let name = tiers::name(FORESTER, plan.tier);
        let Some(size) = plan.ground.size else {
            let n = plan.ground.uneven;
            let cells = if n == 1 { "1 cell".to_string() } else { format!("{n} cells") };
            return format!("{name}\nNeeds flat grass or dirt in front: {cells} in the way (red) · R turns it");
        };
        let patch = ForesterPatch { pos: plan.pos, dir: plan.dir, size };
        let world = &self.sim.world;
        let trees = (0..patch.columns()).filter(|&c| world.get_block(patch.column(c)).is_some_and(is_log)).count();
        let (rows, half) = patch.extent();
        let mut label =
            format!("{name}\n{}×{rows} patch, {} plots · {trees} trees to fell", 2 * half + 1, patch.plots());
        if size < plan.tier {
            let (big_rows, big_half) = ForesterPatch { size: plan.tier, ..patch }.extent();
            label += &format!(" · {}×{big_rows} on flatter ground", 2 * big_half + 1);
        }
        label + " · R turns it"
    }
}
