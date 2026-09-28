//! Placing a multi-block machine (the local player's, presentation only): while one is held and aimed
//! at a face, `footprint_ghost` is where it would stand, each cell marked free or in the way (the host
//! outlines the footprint amber and the cells in the way red: `api/hud.rs`), `footprint_label` names it
//! and says what stops it, and R turns it (`turn_placement`, shared with the quarry). Reads loaded
//! chunks only; never touches the core (placing is `Action::PlaceBlock`, checked again there).

use crate::block::{self, BlockId};
use crate::factory::footprint::{self, Footprint};
use crate::math::{IVec3, Vec3};
use crate::physics::Aabb;
use crate::Game;

/// Where a held machine would go: its block, anchor, facing, and every cell with whether it is in the way.
pub(crate) type Ghost = (BlockId, IVec3, u8, Vec<(IVec3, bool)>);

impl Game {
    /// The block and footprint of the multi-block machine in the selected slot.
    pub(crate) fn held_footprint(&self) -> Option<(BlockId, &'static Footprint)> {
        let stack = self.inventory().selected_stack();
        let block = stack.item.places().filter(|_| !stack.is_empty())?;
        Some((block, footprint::of(block)?))
    }

    /// Where the held multi-block machine would stand if placed now (its anchor against the aimed face).
    /// A cell is in the way if it isn't air or water, or the player stands in it.
    pub(crate) fn footprint_ghost(&self) -> Option<Ghost> {
        let (block, fp) = self.held_footprint()?;
        let hit = self.target.filter(|h| h.normal != IVec3::ZERO)?;
        let (anchor, dir) = (hit.block + hit.normal, self.placing_facing());
        let cells = fp.cells(anchor, dir);
        let body = self.body().aabb();
        let in_body =
            |c: IVec3| Aabb { min: c.as_vec3(), max: c.as_vec3() + Vec3::new(1.0, 1.0, 1.0) }.intersects(&body);
        let blocked = footprint::blocked(&cells, |c| self.sim.world.get_block(c));
        let marked = cells.iter().zip(blocked).map(|(&c, b)| (c, b || in_body(c))).collect();
        Some((block, anchor, dir, marked))
    }

    /// "Assembler" and "2×2×2 · right-click to build · R turns it" on two lines ("" when not placing one).
    pub(crate) fn footprint_label(&self) -> String {
        let Some((block, _, _, cells)) = self.footprint_ghost() else { return String::new() };
        let [w, d, h] = footprint::of(block).map_or([1; 3], |f| f.size);
        let n = cells.iter().filter(|c| c.1).count();
        let state = match n {
            0 => "right-click to build".to_string(),
            1 => "1 cell in the way (red)".to_string(),
            _ => format!("{n} cells in the way (red)"),
        };
        format!("{}\n{w}×{d}×{h} · {state} · R turns it", block::def(block).name)
    }
}
