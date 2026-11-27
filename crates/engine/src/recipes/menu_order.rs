//! The order of the build menu inside each section: `MENU` lists the buildings and tools of the sections that are
//! curated (production, logistics, power, science, tools) in the order a player meets them, grouped by what they
//! do; `menu_order` sorts by it. Every tier of a building (Mk2, Mk3, ...) takes the place of its Mk1, right behind it,
//! so all Mks of one building stand together (the build menu folds them into one tile: `ui/craft-family.ts`; the
//! unplaced items with Mks, scanners, hauler packs, gloves and belts, are listed in `ITEM_TIERS` to do the same). A recipe not listed comes after the listed ones, in recipe order, so a
//! section that is not curated (materials, computing, space, building) simply keeps the recipe order.
//! To place a new recipe: add its item (the Mk1 for a tiered building) to `MENU` where it belongs.

use super::RECIPES;
use crate::block::*;
use crate::factory::tiers;
use crate::item::{
    ItemId, BLADERANG, CORE_DRILL, CRAFT_GLOVES, EXO_FRAME, FLAMETHROWER, GREEN_PACK, HAULER_PACK, HAULER_PACK_MK2,
    HAULER_PACK_MK3, HAULER_PACK_MK4, HAULER_PACK_MK5, HEADLAMP, HOVER_PACK, JETPACK, LINE_HOOK, LOCOMOTIVE,
    PAINT_SPRAYER, PERSONAL_DRONE, PLANNER, RAIL_SIGNAL, RED_PACK, ROPE_ITEM, SCANNER, SCANNER_MK2, SERVO_BOOTS,
    SPRING_BOOTS, TELESCOPE, TOOL_BELT, TOOL_BELT_MK2, TOOL_BELT_MK3, TOOL_BELT_MK4, TOOL_BELT_MK5, WAGON, WORK_GLOVES,
    WORK_GLOVES_MK2, WORK_GLOVES_MK3, WORK_GLOVES_MK4, XRAY_GOGGLES,
};

const fn b(id: BlockId) -> ItemId {
    ItemId::block(id)
}

const MENU: &[ItemId] = &[
    // Production: getting and processing material, then the oil and chemistry chain.
    b(COAL_MINER),
    b(MINER),
    b(QUARRY),
    b(SITE_GRADER),
    b(FORESTER),
    b(THUMPER),
    b(SMELTER),
    b(GLASS_KILN),
    b(BLAST_FURNACE),
    b(ARC_FURNACE),
    b(CRUSHER),
    b(WASHER),
    b(CONSTRUCTOR),
    b(ASSEMBLER),
    b(UPGRADER),
    b(ELECTROLYTIC_CELL),
    b(CENTRIFUGE),
    b(PUMPJACK),
    b(WELLHEAD_SEPARATOR),
    b(GAS_WELL),
    b(REFINERY),
    b(GAS_PLANT),
    b(CRACKER),
    b(CHEMICAL_PLANT),
    b(ELECTROLYSER),
    b(STEAM_REFORMER),
    b(RECYCLER),
    // Logistics: belts and what steers them, storage, fluids, rails, drones, logic and the hoist.
    b(BELT),
    b(SPLITTER),
    b(FILTER),
    b(MERGER),
    b(LIFT),
    b(UNDERPASS_IN),
    b(BELT_EJECTOR),
    b(STORAGE),
    b(SILO),
    b(PUMP),
    b(PIPE),
    b(OUTLET),
    b(RAIL),
    LOCOMOTIVE,
    WAGON,
    RAIL_SIGNAL,
    b(LOADING_DOCK),
    b(UNLOADING_DOCK),
    b(DRONE_PORT),
    b(SENSOR),
    b(WINCH),
    b(HOIST),
    // Power: sources first, then what carries it.
    b(GENERATOR),
    b(BOILER),
    b(TURBINE),
    b(SOLAR_PANEL),
    b(ACCUMULATOR),
    b(WATER_WHEEL),
    b(DIESEL_GENERATOR),
    b(GAS_TURBINE),
    b(REACTOR),
    b(RECTENNA),
    b(POLE),
    b(LONG_POLE),
    b(CABLE),
    // Science.
    b(LAB),
    b(WEATHER_STATION),
    b(RESEARCH_CENTER),
    RED_PACK,
    GREEN_PACK,
    // Tools: the Tool Belt, hand tools, finding things, helpers, flight, then what is worn.
    TOOL_BELT,
    BLADERANG,
    PAINT_SPRAYER,
    FLAMETHROWER,
    SCANNER,
    TELESCOPE,
    CORE_DRILL,
    LINE_HOOK,
    PERSONAL_DRONE,
    PLANNER,
    JETPACK,
    HOVER_PACK,
    HAULER_PACK,
    SPRING_BOOTS,
    SERVO_BOOTS,
    EXO_FRAME,
    WORK_GLOVES,
    HEADLAMP,
    XRAY_GOGGLES,
    ROPE_ITEM,
];

/// Items that come in Mks without being a placed machine: each chain is Mk1 first, and takes one rank in the menu.
const ITEM_TIERS: &[&[ItemId]] = &[
    &[SCANNER, SCANNER_MK2],
    &[HAULER_PACK, HAULER_PACK_MK2, HAULER_PACK_MK3, HAULER_PACK_MK4, HAULER_PACK_MK5],
    &[WORK_GLOVES, WORK_GLOVES_MK2, WORK_GLOVES_MK3, WORK_GLOVES_MK4, CRAFT_GLOVES],
    &[TOOL_BELT, TOOL_BELT_MK2, TOOL_BELT_MK3, TOOL_BELT_MK4, TOOL_BELT_MK5],
];

/// Where recipe `r` sorts inside its section (see the module header).
pub fn menu_order(r: usize) -> usize {
    let item = RECIPES[r].output;
    let (leader, tier) = match tiers::placed_by(item) {
        Some((block, tier)) => (tiers::item_of(block, 0).unwrap_or(item), usize::from(tier)),
        None => {
            ITEM_TIERS.iter().find_map(|c| c.iter().position(|&i| i == item).map(|t| (c[0], t))).unwrap_or((item, 0))
        }
    };
    let rank = MENU
        .iter()
        .position(|&i| i == leader)
        .unwrap_or_else(|| MENU.len() + RECIPES.iter().position(|x| x.output == leader).unwrap_or(r));
    rank * 8 + tier
}

#[cfg(test)]
mod tests;
