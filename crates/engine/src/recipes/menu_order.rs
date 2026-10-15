//! The order of the build menu inside each section: `MENU` lists the buildings and tools of the sections that are
//! curated (production, logistics, power, science, tools) in the order a player meets them, grouped by what they
//! do; `menu_order` sorts by it. Every tier of a building (Mk2, Mk3, ...) takes the place of its Mk1, right behind it,
//! so all Mks of one building stand together. A recipe not listed comes after the listed ones, in recipe order, so a
//! section that is not curated (materials, computing, space, building) simply keeps the recipe order.
//! To place a new recipe: add its item (the Mk1 for a tiered building) to `MENU` where it belongs.

use super::RECIPES;
use crate::block::*;
use crate::factory::tiers;
use crate::item::{
    ItemId, CORE_DRILL, CRAFT_GLOVES, EXO_FRAME, GREEN_PACK, HAULER_PACK, HAULER_PACK_MK2, HAULER_PACK_MK3,
    HAULER_PACK_MK4, HAULER_PACK_MK5, HOVER_PACK, IRON_AXE, IRON_PICKAXE, IRON_SHOVEL, JETPACK, LOCOMOTIVE, MINING_RIG,
    PERSONAL_DRONE, PLANNER, RAIL_SIGNAL, RED_PACK, SCANNER, SCANNER_MK2, SERVO_BOOTS, SPRING_BOOTS, STEEL_AXE,
    STEEL_PICKAXE, STEEL_SHOVEL, STONE_AXE, STONE_PICKAXE, STONE_SHOVEL, WAGON,
};

const fn b(id: BlockId) -> ItemId {
    ItemId::block(id)
}

const MENU: &[ItemId] = &[
    // Production: getting and processing material, then the oil and chemistry chain.
    b(MINER),
    b(QUARRY),
    b(SMELTER),
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
    b(REFINERY),
    b(CRACKER),
    b(CHEMICAL_PLANT),
    b(ELECTROLYSER),
    b(RECYCLER),
    // Logistics: belts and what steers them, storage, fluids, rails, drones, logic and the hoist.
    b(BELT),
    b(SPLITTER),
    b(FILTER),
    b(LIFT),
    b(UNDERPASS_IN),
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
    b(REACTOR),
    b(RECTENNA),
    b(POLE),
    b(LONG_POLE),
    b(CABLE),
    // Science.
    b(LAB),
    b(RESEARCH_CENTER),
    RED_PACK,
    GREEN_PACK,
    // Tools: hand tools, finding things, helpers, flight, then what is worn.
    STONE_PICKAXE,
    STONE_AXE,
    STONE_SHOVEL,
    IRON_PICKAXE,
    IRON_AXE,
    IRON_SHOVEL,
    STEEL_PICKAXE,
    STEEL_AXE,
    STEEL_SHOVEL,
    SCANNER,
    SCANNER_MK2,
    CORE_DRILL,
    PERSONAL_DRONE,
    PLANNER,
    JETPACK,
    HOVER_PACK,
    HAULER_PACK,
    HAULER_PACK_MK2,
    HAULER_PACK_MK3,
    HAULER_PACK_MK4,
    HAULER_PACK_MK5,
    SPRING_BOOTS,
    SERVO_BOOTS,
    EXO_FRAME,
    MINING_RIG,
    CRAFT_GLOVES,
];

/// Where recipe `r` sorts inside its section (see the module header).
pub fn menu_order(r: usize) -> usize {
    let item = RECIPES[r].output;
    let (leader, tier) = match tiers::placed_by(item) {
        Some((block, tier)) => (tiers::item_of(block, 0).unwrap_or(item), usize::from(tier)),
        None => (item, 0),
    };
    let rank = MENU
        .iter()
        .position(|&i| i == leader)
        .unwrap_or_else(|| MENU.len() + RECIPES.iter().position(|x| x.output == leader).unwrap_or(r));
    rank * 8 + tier
}

#[cfg(test)]
mod tests;
