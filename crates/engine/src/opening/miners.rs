//! The opening's guide past the line: once the ingots are taken, two coal miners, one set against the starter coal
//! patch and lit, one against the starter iron patch and fuelled. The scene paid for both (`STOCK`, `OUT_STOCK`).
//!
//! Invariants: read-only on the core; every step comes from what stands and what the player holds, so doing things
//! in another order (a miner on iron first, a third one, a miner set on any coal) only skips steps. Any coal miner on
//! a coal deposit counts, not only one on the marked patch. Scenes without patches (before version 67) show none.
//! To change a step: its text in `Step::text`, its condition in `mining_step`, its prompt in `mining_marker`.

use crate::block::{BlockId, AIR, COAL_MINER, COAL_ORE, IRON_ORE};
use crate::factory::Factory;
use crate::inventory::Inventory;
use crate::item::ItemId;
use crate::math::IVec3;
use crate::world::World;

use super::{Opening, Step, PROMPT_COAL, PROMPT_IRON, PROMPT_OPEN};

impl Opening {
    /// The guide's step: craft, set on coal, light, set on iron, fuel.
    pub(super) fn mining_step(&self, inv: &Inventory, f: &Factory) -> Step {
        if self.coal.is_none() {
            return Step::Done;
        }
        let held = inv.count(COAL_MINER.into()) > 0;
        let place = match (on(f, COAL_ORE), on(f, IRON_ORE)) {
            (None, _) => Step::PlaceCoal,
            (Some((_, false)), _) => return Step::FuelCoal,
            (Some(_), None) => Step::PlaceIron,
            (Some(_), Some((_, false))) => return Step::FuelIron,
            (Some(_), Some(_)) => return Step::Done,
        };
        if held {
            place
        } else {
            Step::Craft
        }
    }

    /// Whether both coal miners stand and have been given fuel: the opening is over.
    pub(super) fn miners_done(&self, f: &Factory) -> bool {
        self.coal.is_some() && on(f, COAL_ORE).is_some_and(|m| m.1) && on(f, IRON_ORE).is_some_and(|m| m.1)
    }

    /// The prompt for a guide step: over the patch to set a miner on, or on the miner to fuel.
    pub(super) fn mining_marker(&self, step: Step, f: &Factory) -> Option<(IVec3, u8)> {
        match step {
            Step::PlaceCoal => self.coal.map(|p| (p, PROMPT_COAL)),
            Step::PlaceIron => self.iron.map(|p| (p, PROMPT_IRON)),
            Step::FuelCoal => on(f, COAL_ORE).map(|m| (m.0, PROMPT_OPEN)),
            Step::FuelIron => on(f, IRON_ORE).map(|m| (m.0, PROMPT_OPEN)),
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

/// A coal miner on a deposit of `ore` and whether it was lit, a lit one first.
fn on(f: &Factory, ore: BlockId) -> Option<(IVec3, bool)> {
    let ore = ItemId::block(ore);
    let mut found = f.coal_miners().filter(|m| m.ore == ore).map(|m| (m.pos, m.lit));
    let first = found.next()?;
    Some(if first.1 { first } else { found.find(|m| m.1).unwrap_or(first) })
}
