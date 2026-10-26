//! Recipes as data tables: hand crafting ([`RECIPES`], listed in order by the build menu) here; what
//! machines make, by category, and fuels in `machine.rs`. Research locks recipes (`research.rs`).
//! Content lint for every table: `tests.rs`.
//!
//! To add a hand recipe: a `pub const` in its era's or set's file, named in `RECIPES` (its `group` is its build-menu
//! section); the first machines', tools' and wood's are in `first.rs`, tier items' are in `tiers.rs`, the
//! heavy machines' in `heavy.rs`, Electronics' in `electronics.rs`, cables' in `wiring.rs`, solar power's in
//! `solar.rs`, the upgrader's (and its machine row per tier) in `refit.rs`, the automated buildings' machine rows in `automate.rs`, the chip fab's in `compute.rs`, the launch site's (and the rocket parts' machine rows) in `orbit.rs`, the recycler's in `recycler.rs` (what it pays: `recycling.rs`), the electrolytic cell's in `aluminium.rs`, the pumpjack's and canisters' in `chemistry.rs`, the railway's in `transport.rs`, plates, rods, screws and wire in `materials.rs`. How long a craft takes by hand: `timing.rs`.

use crate::block::*;
use crate::item::ItemId;

mod aluminium;
mod automate;
mod chemistry;
mod compute;
pub mod content;
mod decor;
mod electronics;
mod first;
mod gear;
mod gold;
mod group;
mod heavy;
mod house;
mod machine;
mod machine_later;
mod materials;
mod menu_order;
mod orbit;
mod recycler;
pub mod recycling;
mod refit;
mod rows;
mod scrap;
mod solar;
mod tiers;
mod timing;
mod tooling;
mod towns;
mod transport;
mod wiring;
use aluminium::*;
use chemistry::*;
use compute::*;
use electronics::*;
use first::*;
use gear::*;
use gold::*;
pub use group::{Group, GROUPS};
use heavy::*;
use house::*;
pub use machine::*;
use materials::*;
pub use menu_order::menu_order;
use orbit::{GROUND_STATION_RECIPE, LANDING_PAD_RECIPE, LAUNCH_SITE_RECIPE, POD_LAUNCHER_RECIPE, RECTENNA_RECIPE};
use recycler::*;
use refit::UPGRADER_RECIPE;
pub use rows::*;
use solar::*;
use tiers::*;
#[cfg(test)]
use timing::{BASE_TICKS, MAX_TICKS};
use tooling::*;
use towns::TOWN_DEPOT_RECIPE;
use transport::*;
use wiring::*;

pub struct Recipe {
    pub output: ItemId,
    /// Its section in the build menu.
    pub group: Group,
    pub count: u32,
    pub inputs: &'static [(ItemId, u32)],
    /// One line for the build menu.
    pub blurb: &'static str,
}

pub const RECIPES: &[Recipe] = &[
    MINER_RECIPE,
    BELT_RECIPE,
    STORAGE_RECIPE,
    SMELTER_RECIPE,
    CONSTRUCTOR_RECIPE,
    SPLITTER_RECIPE,
    FILTER_RECIPE,
    LIFT_RECIPE,
    UNDERPASS_RECIPE,
    BELT_EJECTOR_RECIPE,
    PUMP_RECIPE,
    PIPE_RECIPE,
    OUTLET_RECIPE,
    GENERATOR_RECIPE,
    POLE_RECIPE,
    LAB_RECIPE,
    RED_PACK_RECIPE,
    GREEN_PACK_RECIPE,
    GREEN_KIT_RECIPE,
    // A tier's item is the tier below plus its kits (the lint checks these match `factory/tiers.rs`).
    MINER_MK2_RECIPE,
    SMELTER_MK2_RECIPE,
    CONSTRUCTOR_MK2_RECIPE,
    ASSEMBLER_RECIPE,
    BLAST_FURNACE_RECIPE,
    MINER_MK3_RECIPE,
    SMELTER_MK3_RECIPE,
    CONSTRUCTOR_MK3_RECIPE,
    ASSEMBLER_MK2_RECIPE,
    ASSEMBLER_MK3_RECIPE,
    BLAST_FURNACE_MK2_RECIPE,
    BLAST_FURNACE_MK3_RECIPE,
    MINER_MK4_RECIPE,
    SMELTER_MK4_RECIPE,
    CONSTRUCTOR_MK4_RECIPE,
    ASSEMBLER_MK4_RECIPE,
    BLAST_FURNACE_MK4_RECIPE,
    QUARRY_RECIPE,
    BELT_MK2_RECIPE,
    BELT_MK3_RECIPE,
    BELT_MK4_RECIPE,
    UNDERPASS_MK2_RECIPE,
    UNDERPASS_MK3_RECIPE,
    UNDERPASS_MK4_RECIPE,
    POLE_MK2_RECIPE,
    POLE_MK3_RECIPE,
    POLE_MK4_RECIPE,
    CABLE_RECIPE,
    BOX_MK2_RECIPE,
    BOX_MK3_RECIPE,
    PUMP_MK2_RECIPE,
    PUMP_MK3_RECIPE,
    QUARRY_MK2_RECIPE,
    QUARRY_MK3_RECIPE,
    LAB_MK2_RECIPE,
    LAB_MK3_RECIPE,
    LAB_MK4_RECIPE,
    GENERATOR_MK2_RECIPE,
    BOILER_RECIPE,
    TURBINE_RECIPE,
    CRUSHER_RECIPE,
    SILO_RECIPE,
    ARC_FURNACE_RECIPE,
    ELECTROLYTIC_CELL_RECIPE,
    RAIL_RECIPE,
    LOCOMOTIVE_RECIPE,
    WAGON_RECIPE,
    RAIL_SIGNAL_RECIPE,
    LOADING_DOCK_RECIPE,
    UNLOADING_DOCK_RECIPE,
    STONE_PICKAXE_RECIPE,
    STONE_AXE_RECIPE,
    STONE_SHOVEL_RECIPE,
    IRON_PICKAXE_RECIPE,
    IRON_AXE_RECIPE,
    IRON_SHOVEL_RECIPE,
    STEEL_PICKAXE_RECIPE,
    STEEL_AXE_RECIPE,
    STEEL_SHOVEL_RECIPE,
    SCANNER_RECIPE,
    CORE_DRILL_RECIPE,
    LAMP_RECIPE,
    TORCH_RECIPE,
    PLANKS_RECIPE,
    STICK_RECIPE,
    LADDER_RECIPE,
    IRON_PLATE_RECIPE,
    IRON_ROD_RECIPE,
    SCREW_RECIPE,
    COPPER_WIRE_RECIPE,
    SOLAR_PANEL_RECIPE,
    ACCUMULATOR_RECIPE,
    SCANNER_MK2_RECIPE,
    SENSOR_RECIPE,
    DRONE_PORT_RECIPE,
    DRONE_PORT_MK2_RECIPE,
    DRONE_PORT_MK3_RECIPE,
    DRONE_PORT_MK4_RECIPE,
    JETPACK_RECIPE,
    PERSONAL_DRONE_RECIPE,
    PLANNER_RECIPE,
    HAULER_PACK_RECIPE,
    HAULER_PACK_MK2_RECIPE,
    HAULER_PACK_MK3_RECIPE,
    HAULER_PACK_MK4_RECIPE,
    HAULER_PACK_MK5_RECIPE,
    SPRING_BOOTS_RECIPE,
    SERVO_BOOTS_RECIPE,
    EXO_FRAME_RECIPE,
    MINING_RIG_RECIPE,
    HOVER_PACK_RECIPE,
    PUMPJACK_RECIPE,
    CANISTER_RECIPE,
    REFINERY_RECIPE,
    CRACKER_RECIPE,
    CHEMICAL_PLANT_RECIPE,
    DIESEL_GENERATOR_RECIPE,
    ELECTROLYSER_RECIPE,
    WASHER_RECIPE,
    RESEARCH_CENTER_RECIPE,
    RESEARCH_CENTER_MK2_RECIPE,
    RESEARCH_CENTER_MK3_RECIPE,
    RESEARCH_CENTER_MK4_RECIPE,
    WATER_WHEEL_RECIPE,
    HOIST_RECIPE,
    WINCH_RECIPE,
    CENTRIFUGE_RECIPE,
    REACTOR_RECIPE,
    MINER_MK5_RECIPE,
    SMELTER_MK5_RECIPE,
    CONSTRUCTOR_MK5_RECIPE,
    ASSEMBLER_MK5_RECIPE,
    BLAST_FURNACE_MK5_RECIPE,
    DRONE_PORT_MK5_RECIPE,
    RESEARCH_CENTER_MK5_RECIPE,
    RECYCLER_RECIPE,
    CHIP_FAB_RECIPE,
    FIBRE_NODE_RECIPE,
    DATACENTER_RECIPE,
    COOLING_TOWER_RECIPE,
    AI_LAB_RECIPE,
    OPTIMIZER_RECIPE,
    LASER_EMITTER_RECIPE,
    LASER_RECEIVER_RECIPE,
    LASER_MIRROR_RECIPE,
    DATA_RECEIVER_RECIPE,
    SWARM_HUB_RECIPE,
    CRAFT_GLOVES_RECIPE,
    LAUNCH_SITE_RECIPE,
    GROUND_STATION_RECIPE,
    RECTENNA_RECIPE,
    LONG_POLE_RECIPE,
    POD_LAUNCHER_RECIPE,
    LANDING_PAD_RECIPE,
    UPGRADER_RECIPE,
    TOWN_DEPOT_RECIPE,
    TELESCOPE_RECIPE,
    SICKLE_RECIPE,
    COAL_MINER_RECIPE,
    WOOD_DOOR_RECIPE,
    IRON_DOOR_RECIPE,
    WINDOW_RECIPE,
    PLASTER_RECIPE,
    TIMBER_FRAME_RECIPE,
    WOOD_FLOOR_RECIPE,
    THATCH_RECIPE,
    RED_ROOF_RECIPE,
    SLATE_ROOF_RECIPE,
    GREEN_ROOF_RECIPE,
    CHIMNEY_BRICKS_RECIPE,
];

/// The item that is block `id`, to keep the tables short.
const fn b(id: BlockId) -> ItemId {
    ItemId::block(id)
}

#[cfg(test)]
mod tests;
