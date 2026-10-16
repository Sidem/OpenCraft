//! The highland workshop's perk (step 13.4). While a workshop is supplied, the gear in `GEAR` costs half its inputs
//! (rounded up) in hand crafting anywhere (`crafting.rs` asks `Settlements::gear_works` and costs each craft by
//! `gear_cost`), and worn tools put into its depot come back out whole, one a batch.
//! To make more gear cheaper: its item in `GEAR`.

use crate::factory::Buffer;
use crate::item::{
    ItemId, CRAFT_GLOVES, HAULER_PACK, HAULER_PACK_MK2, HAULER_PACK_MK3, HAULER_PACK_MK4, HAULER_PACK_MK5, JETPACK,
};
use crate::recipes::Recipe;
use crate::tools::{ToolDef, ToolKind, TOOLS};

/// What costs half while a workshop is on: hauler packs of every mark, jetpacks and crafting gloves.
const GEAR: [ItemId; 7] =
    [HAULER_PACK, HAULER_PACK_MK2, HAULER_PACK_MK3, HAULER_PACK_MK4, HAULER_PACK_MK5, JETPACK, CRAFT_GLOVES];

/// Whether hand-crafting `output` costs half while a workshop is on.
pub fn discounted(output: ItemId) -> bool {
    GEAR.contains(&output)
}

/// What one craft of `r` takes: its inputs, or half of each (rounded up) when `half` and it is workshop gear.
pub fn gear_cost(r: &Recipe, half: bool) -> impl Iterator<Item = (ItemId, u32)> + '_ {
    let half = half && discounted(r.output);
    r.inputs.iter().map(move |&(item, n)| (item, if half { n.div_ceil(2) } else { n }))
}

/// The tools that wear (pickaxes, axes and shovels), each up to one whole tool's uses.
pub(super) fn wants() -> Vec<(ItemId, u32)> {
    TOOLS.iter().filter(|t| wears(t)).map(|t| (t.item, t.tier.uses)).collect()
}

/// One batch: the first worn tool held (up to one whole tool's uses) comes out whole, if there is room for it.
pub(super) fn repair(input: &mut Buffer, out: &mut Buffer) {
    for t in TOOLS.iter().filter(|t| wears(t)) {
        let held = input.count(t.item).min(t.tier.uses);
        if held > 0 && out.can_hold(t.item, t.tier.uses) {
            input.remove(t.item, held);
            out.add(t.item, t.tier.uses);
            return;
        }
    }
}

fn wears(t: &ToolDef) -> bool {
    matches!(t.kind, ToolKind::Pickaxe | ToolKind::Axe | ToolKind::Shovel)
}
