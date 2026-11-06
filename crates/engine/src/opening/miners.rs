//! The opening's guide past the line: once the ingots are taken, two coal miners, one set against the starter coal
//! patch, lit and given a box to fill (the line's first box, empty by then), one against the starter iron patch and
//! fuelled. The scene paid for both (`STOCK`, `OUT_STOCK`).
//!
//! Invariants: read-only on the core; every step comes from what stands and what the player holds, so doing things
//! in another order (a miner on iron first, a third one, a miner set on any coal) only skips steps. Any coal miner on
//! a coal deposit counts, not only one on the marked patch. Scenes without patches (before version 67) show none.
//! To change a step: its text in `Step::text`, its condition in `mining_step`, its prompt in `mining_marker`.

use crate::block::{BlockId, AIR, COAL_MINER, COAL_ORE, IRON_ORE, SOLID};
use crate::factory::{CoalMinerView, Factory, DIRS};
use crate::inventory::Inventory;
use crate::item::ItemId;
use crate::math::IVec3;
use crate::world::World;

use super::{Opening, Step, PROMPT_COAL, PROMPT_HOLD_LEFT, PROMPT_IRON, PROMPT_OPEN, PROMPT_RIGHT_CLICK};

impl Opening {
    /// The guide's step: craft, set on coal, light, give it a box, craft the second, set on iron, fuel.
    pub(super) fn mining_step(&self, inv: &Inventory, f: &Factory) -> Step {
        if self.coal.is_none() {
            return Step::Done;
        }
        let held = inv.count(COAL_MINER.into()) > 0;
        let (place, craft) = match (on(f, COAL_ORE), on(f, IRON_ORE)) {
            (None, _) => (Step::PlaceCoal, Step::Craft),
            (Some(c), _) if !c.lit => return Step::FuelCoal,
            (Some(c), None) if !c.feeds => return Step::BoxCoal,
            (Some(_), None) => (Step::PlaceIron, Step::CraftIron),
            (Some(_), Some(i)) if !i.lit => return Step::FuelIron,
            (Some(_), Some(_)) => return Step::Done,
        };
        if held {
            place
        } else {
            craft
        }
    }

    /// Whether both coal miners stand and have been given fuel: the opening is over.
    pub(super) fn miners_done(&self, f: &Factory) -> bool {
        self.coal.is_some() && on(f, COAL_ORE).is_some_and(|m| m.lit) && on(f, IRON_ORE).is_some_and(|m| m.lit)
    }

    /// The prompt for a guide step: over the patch to set a miner on, on the miner to fuel, or for the box: on the
    /// line's first box while it stands (take it), then on the coal miner (the host moves it to the ground beside it,
    /// `box_spot`, which needs the world).
    pub(super) fn mining_marker(&self, step: Step, f: &Factory) -> Option<(IVec3, u8)> {
        match step {
            Step::PlaceCoal => self.coal.map(|p| (p, PROMPT_COAL)),
            Step::PlaceIron => self.iron.map(|p| (p, PROMPT_IRON)),
            Step::FuelCoal => on(f, COAL_ORE).map(|m| (m.pos, PROMPT_OPEN)),
            Step::FuelIron => on(f, IRON_ORE).map(|m| (m.pos, PROMPT_OPEN)),
            Step::BoxCoal if f.box_slots(self.source).is_some() => Some((self.source, PROMPT_HOLD_LEFT)),
            Step::BoxCoal => on(f, COAL_ORE).map(|m| (m.pos, PROMPT_RIGHT_CLICK)),
            _ => None,
        }
    }
}

/// How far round a patch's middle `open_top` looks.
const OPEN_REACH: i32 = 3;

/// The block of `ore` nearest `top` (a starter patch's top block) with open air above it, so the marker is not hidden
/// under a tree and a miner fits on it: the top block of each column within `OPEN_REACH`. `top` itself if none is.
pub(super) fn open_top(world: &mut World, top: IVec3, ore: BlockId) -> IVec3 {
    let mut best = (i32::MAX, top);
    for dx in -OPEN_REACH..=OPEN_REACH {
        for dz in -OPEN_REACH..=OPEN_REACH {
            let column = top + IVec3::new(dx, 0, dz);
            for dy in [1, 0, -1, -2] {
                let cell = column + IVec3::new(0, dy, 0);
                let open = world.block_anywhere_or_generate(cell + IVec3::new(0, 1, 0)) == AIR;
                if open && world.block_anywhere_or_generate(cell) == ore {
                    let d = dx * dx + dz * dz + dy * dy;
                    if d < best.0 {
                        best = (d, cell);
                    }
                    break;
                }
            }
        }
    }
    best.1
}

/// Where a box beside the coal miner at `miner` would go: an open cell beside it on solid ground (the player
/// right-clicks the ground below it; a click on the miner opens it instead). None when every side is blocked.
pub(crate) fn box_spot(world: &World, miner: IVec3) -> Option<IVec3> {
    let up = IVec3::new(0, 1, 0);
    let open = |c: IVec3| world.block_anywhere(c) == Some(AIR);
    let ground = |c: IVec3| world.block_anywhere(c - up).is_some_and(|b| SOLID[b as usize]);
    DIRS.iter().map(|&d| miner + d).find(|&c| open(c) && ground(c))
}

/// A coal miner on a deposit of `ore`, a lit one first, and of those one that feeds a box or belt.
pub(crate) fn on(f: &Factory, ore: BlockId) -> Option<CoalMinerView> {
    let ore = ItemId::block(ore);
    f.coal_miners().filter(|m| m.ore == ore).max_by_key(|m| (m.lit, m.feeds))
}
