//! What the blueprint library shows of a blueprint (presentation only): a top-down plan of its pieces and
//! the items building it takes. `api/blueprint.rs` hands both to `web/src/ui/blueprint-plan.ts`, which draws
//! the plan. To show more of a piece: append a number to [`PLAN_FIELDS`] and push it in [`Blueprint::plan`].

use super::Blueprint;
use crate::factory::{self, Kind};
use crate::math::IVec3;

/// Numbers per piece in [`Blueprint::plan`].
pub const PLAN_FIELDS: usize = 8;

impl Blueprint {
    /// The box size (x, y, z), then each piece, lowest first: `[x0, z0, x1, z1, top, item, facing, flow]`, the
    /// columns its footprint covers (inclusive, from the box's corner), its highest cell, the item it is built from,
    /// its facing (0 north, 1 east, 2 south, 3 west) and 1 for belts and routers, whose facing is the way items go.
    pub fn plan(&self) -> Vec<i32> {
        let mut pieces: Vec<[i32; PLAN_FIELDS]> = self
            .ghosts_at(IVec3::ZERO, 0)
            .iter()
            .map(|g| {
                let cells = g.cells();
                let lo = |f: fn(&IVec3) -> i32| cells.iter().map(f).min().unwrap_or(0);
                let hi = |f: fn(&IVec3) -> i32| cells.iter().map(f).max().unwrap_or(0);
                let flow = factory::machine(g.block).is_some_and(|m| matches!(m.kind, Kind::Belt | Kind::Router));
                let item = i32::from(g.item().0);
                [
                    lo(|c| c.x),
                    lo(|c| c.z),
                    hi(|c| c.x),
                    hi(|c| c.z),
                    hi(|c| c.y),
                    item,
                    i32::from(g.facing),
                    i32::from(flow),
                ]
            })
            .collect();
        // Lowest first, so a piece drawn later covers what lies under it.
        crate::math::sort_small_by_key(&mut pieces, |p| p[4]);
        let mut out = vec![self.size.x, self.size.y, self.size.z];
        out.extend(pieces.iter().flatten());
        out
    }
}
