//! The basalt forge's perk (step 13.4): raw iron and copper ore in, iron plates and copper ingots out, half again
//! what smelting gives (a smelter turns 4 iron ore into 2 plates' worth of ingots, the forge into 3 plates).
//! To add an ore: a row in `ROWS`.

use crate::block::{COPPER_ORE, IRON_ORE};
use crate::factory::Buffer;
use crate::item::{stack_size, ItemId, COPPER_INGOT, IRON_PLATE};
use crate::sim::SimEvent;

use super::BATCH;

/// Ore in, what a batch of `BATCH` ore gives.
const ROWS: [(ItemId, ItemId, u32); 2] =
    [(ItemId::block(IRON_ORE), IRON_PLATE, 3), (ItemId::block(COPPER_ORE), COPPER_INGOT, 6)];
/// Most stacks of each ore its depot holds.
const ORE_STACKS: u32 = 3;

/// Each ore, up to a minute's worth at its stage (at most `ORE_STACKS` stacks).
pub(super) fn wants(built: u8) -> Vec<(ItemId, u32)> {
    ROWS.iter().map(|r| (r.0, super::rate(built).min(ORE_STACKS * stack_size(r.0)))).collect()
}

/// One batch: the first ore (taking turns by `n`, the batch's number) with a batch held and room for what it gives.
pub(super) fn batch(input: &mut Buffer, out: &mut Buffer, n: u64, events: &mut Vec<SimEvent>) {
    for k in 0..ROWS.len() {
        let (ore, product, count) = ROWS[(n as usize + k) % ROWS.len()];
        if input.count(ore) >= BATCH && out.can_hold(product, count) {
            input.remove(ore, BATCH);
            out.add(product, count);
            events.push(SimEvent::Consumed { item: ore, count: BATCH });
            events.push(SimEvent::Produced { item: product, count });
            return;
        }
    }
}
