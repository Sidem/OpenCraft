//! The highland workshop's perk (step 13.4). While a workshop is supplied, the gear in `GEAR` costs half its inputs
//! (rounded up) in hand crafting anywhere (`crafting.rs` asks `Settlements::gear_works` and costs each craft by
//! `gear_cost`). It takes nothing in (it repaired worn tools until the Tool Belt ended wear, save 87).
//! To make more gear cheaper: its item in `GEAR`.

use crate::item::{
    ItemId, CRAFT_GLOVES, HAULER_PACK, HAULER_PACK_MK2, HAULER_PACK_MK3, HAULER_PACK_MK4, HAULER_PACK_MK5, JETPACK,
    TOOL_BELT, TOOL_BELT_MK2, TOOL_BELT_MK3, TOOL_BELT_MK4, TOOL_BELT_MK5,
};
use crate::recipes::Recipe;

/// What costs half while a workshop is on: hauler packs and Tool Belts of every mark, jetpacks and crafting gloves.
const GEAR: [ItemId; 12] = [
    HAULER_PACK,
    HAULER_PACK_MK2,
    HAULER_PACK_MK3,
    HAULER_PACK_MK4,
    HAULER_PACK_MK5,
    TOOL_BELT,
    TOOL_BELT_MK2,
    TOOL_BELT_MK3,
    TOOL_BELT_MK4,
    TOOL_BELT_MK5,
    JETPACK,
    CRAFT_GLOVES,
];

/// Whether hand-crafting `output` costs half while a workshop is on.
pub fn discounted(output: ItemId) -> bool {
    GEAR.contains(&output)
}

/// What one craft of `r` takes: its inputs, or half of each (rounded up) when `half` and it is workshop gear.
pub fn gear_cost(r: &Recipe, half: bool) -> impl Iterator<Item = (ItemId, u32)> + '_ {
    let half = half && discounted(r.output);
    r.inputs.iter().map(move |&(item, n)| (item, if half { n.div_ceil(2) } else { n }))
}
