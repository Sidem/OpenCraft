//! Recipes as data tables: hand crafting ([`RECIPES`], listed in order by the build menu) here; what
//! machines make, by category, and fuels in `machine.rs`. Research locks recipes (`research.rs`).
//! Content lint for every table: `tests.rs`.
//!
//! To add a hand recipe: add a row (its `group` is its build-menu section); tier items' are in `tiers.rs`, the
//! heavy machines' in `heavy.rs`, Electronics' in `electronics.rs`, cables' in `wiring.rs`, solar power's in
//! `solar.rs`, plates, rods, screws and wire in `materials.rs`. How long a craft takes by hand: `timing.rs`.

use crate::block::*;
use crate::item::{
    ItemId, COPPER_INGOT, COPPER_WIRE, CORE_DRILL, GEAR, GREEN_KIT, GREEN_PACK, IRON_INGOT, IRON_PLATE, IRON_ROD,
    MOTOR, RED_PACK, SCANNER, SCREW, STICK,
};

mod electronics;
mod gear;
mod heavy;
mod machine;
mod materials;
mod solar;
mod tiers;
mod timing;
mod tooling;
mod wiring;
use electronics::*;
use gear::*;
use heavy::*;
pub use machine::*;
use materials::*;
use solar::*;
use tiers::*;
#[cfg(test)]
use timing::{BASE_TICKS, MAX_TICKS};
use tooling::*;
use wiring::*;

/// The build menu's sections, in the order it shows them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Group {
    Materials,
    Production,
    Logistics,
    Power,
    Science,
    Tools,
    Building,
}

pub const GROUPS: [Group; 7] = [
    Group::Materials,
    Group::Production,
    Group::Logistics,
    Group::Power,
    Group::Science,
    Group::Tools,
    Group::Building,
];

impl Group {
    pub fn name(self) -> &'static str {
        match self {
            Group::Materials => "Materials",
            Group::Production => "Production",
            Group::Logistics => "Logistics",
            Group::Power => "Power",
            Group::Science => "Science",
            Group::Tools => "Tools",
            Group::Building => "Building",
        }
    }
}

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
    Recipe {
        output: b(MINER),
        group: Group::Production,
        count: 1,
        inputs: &[(IRON_PLATE, 5), (IRON_ROD, 4), (COPPER_WIRE, 6)],
        blurb: "Place it against an ore block. It drills the whole deposit, recovers 60% of what it draws, \
                and pushes ore into a belt, box, smelter or generator beside it. Needs power (5 kW).",
    },
    Recipe {
        output: b(BELT),
        group: Group::Logistics,
        count: 4,
        inputs: &[(IRON_PLATE, 1), (IRON_ROD, 1)],
        blurb: "Carries items. Hold right-click and drag to lay a line; it climbs and drops one-block steps \
                by itself. Belts feed belts, machines, and belts from the side. Press R on a belt to turn it.",
    },
    Recipe {
        output: b(STORAGE),
        group: Group::Logistics,
        count: 1,
        inputs: &[(b(PLANKS), 8), (IRON_PLATE, 2)],
        blurb: "Holds 24 stacks. Belts deliver into it; a belt leading away from it is fed from it. \
                Right-click to empty it into your inventory.",
    },
    Recipe {
        output: b(SMELTER),
        group: Group::Production,
        count: 1,
        inputs: &[(b(STONE), 16)],
        blurb: "A stone furnace. Melts iron or copper ore into ingots, sand into glass (and, with Masonry, stone into bricks) while \
                it has fuel: coal ore or logs. Belts bring both in; a belt leading away takes the ingots. \
                Right-click to open it.",
    },
    Recipe {
        output: b(CONSTRUCTOR),
        group: Group::Production,
        count: 1,
        inputs: &[(IRON_INGOT, 10), (COPPER_INGOT, 4), (b(STONE), 8)],
        blurb: "Shapes ingots into parts: plates, rods, screws and wire. Right-click to choose what it makes; \
                belts bring the ingots in and take the parts away.",
    },
    Recipe {
        output: b(SPLITTER),
        group: Group::Logistics,
        count: 1,
        inputs: &[(IRON_PLATE, 2), (b(BELT), 2)],
        blurb: "Takes items from belts leading into it and shares them between the belts leading away in \
                front, to the left and to the right.",
    },
    Recipe {
        output: b(FILTER),
        group: Group::Logistics,
        count: 1,
        inputs: &[(IRON_PLATE, 2), (COPPER_WIRE, 2), (b(BELT), 2)],
        blurb: "Sends the item you choose straight on and everything else to the left and right. \
                Right-click to choose the item.",
    },
    Recipe {
        output: b(LIFT),
        group: Group::Logistics,
        count: 2,
        inputs: &[(IRON_ROD, 2), (b(BELT), 2)],
        blurb: "Carries items straight up. Stack lifts facing the same way to climb higher; the top one \
                hands items on one block ahead and one up. You can climb a lift stack like a ladder.",
    },
    Recipe {
        output: b(UNDERPASS_IN),
        group: Group::Logistics,
        count: 1,
        inputs: &[(IRON_PLATE, 2), (b(BELT), 2)],
        blurb: "Takes items under whatever is in front of it to an underpass exit facing the same way, \
                up to 5 blocks ahead. Lets belts cross.",
    },
    Recipe {
        output: b(UNDERPASS_OUT),
        group: Group::Logistics,
        count: 1,
        inputs: &[(IRON_PLATE, 2), (b(BELT), 2)],
        blurb: "Where items come back up from an underpass entry behind it, then carry on like a belt.",
    },
    Recipe {
        output: b(PUMP),
        group: Group::Logistics,
        count: 1,
        inputs: &[(IRON_PLATE, 6), (IRON_ROD, 4), (COPPER_WIRE, 6)],
        blurb: "Lifts 2 blocks of still water a second out of the water it touches, highest first. Pipe it to \
                an outlet; needs 5 kW. It can't lower the sea.",
    },
    Recipe {
        output: b(PIPE),
        group: Group::Logistics,
        count: 4,
        inputs: &[(IRON_PLATE, 2)],
        blurb: "Joins pumps to outlets. Pipes connect on every side.",
    },
    Recipe {
        output: b(OUTLET),
        group: Group::Logistics,
        count: 1,
        inputs: &[(IRON_PLATE, 3), (IRON_ROD, 2)],
        blurb: "Pours the water its pipes bring out in front of it (it faces the way you do), filling from the \
                bottom up.",
    },
    Recipe {
        output: b(GENERATOR),
        group: Group::Power,
        count: 1,
        inputs: &[(IRON_PLATE, 4), (COPPER_WIRE, 8), (b(STONE), 12)],
        blurb: "Burns coal ore (270 kJ) or logs (135 kJ) into up to 60 kW, and only as much as its grid uses. \
                Place a power pole within 5 blocks; a belt or a miner beside it brings fuel. Right-click to open it.",
    },
    Recipe {
        output: b(POLE),
        group: Group::Power,
        count: 2,
        inputs: &[(STICK, 2), (COPPER_WIRE, 2)],
        blurb: "Links to every pole within 10 blocks and powers generators and machines within 5. \
                Miners, constructors, splitters, filters, labs, pumps and quarries need power.",
    },
    Recipe {
        output: b(LAB),
        group: Group::Science,
        count: 1,
        inputs: &[(IRON_PLATE, 6), (COPPER_WIRE, 8), (b(BELT), 4)],
        blurb: "Uses science packs to research new machines (press T to choose what). Belts bring packs in; \
                needs power.",
    },
    Recipe {
        output: RED_PACK,
        group: Group::Science,
        count: 1,
        inputs: &[(IRON_PLATE, 1), (COPPER_WIRE, 2)],
        blurb: "A lab uses these to research the first techs.",
    },
    Recipe {
        output: GREEN_PACK,
        group: Group::Science,
        count: 1,
        inputs: &[(b(BELT), 2), (SCREW, 4)],
        blurb: "A lab uses these, with red packs, for later techs.",
    },
    Recipe {
        output: GREEN_KIT,
        group: Group::Production,
        count: 4,
        inputs: &[(GEAR, 2), (SCREW, 4), (COPPER_WIRE, 2)],
        blurb: "Upgrades machines to Mk2 (green stripe) in place. Hold kits and right-click a miner, smelter or \
                constructor (4 kits), or hold the button and drag along belts (1 kit each).",
    },
    // A tier's item is the tier below plus its kits (the lint checks these match `factory/tiers.rs`).
    MINER_MK2_RECIPE,
    SMELTER_MK2_RECIPE,
    CONSTRUCTOR_MK2_RECIPE,
    Recipe {
        output: b(ASSEMBLER),
        group: Group::Production,
        count: 1,
        inputs: &[(IRON_PLATE, 12), (GEAR, 6), (COPPER_WIRE, 12), (IRON_ROD, 4)],
        blurb: "Puts parts together: motors, concrete, packs and kits. It is 2×2×2 (R turns it before you place \
                it). Belts bring parts in at the hatches on its back and sides; the front hatch gives. 20 kW.",
    },
    Recipe {
        output: b(BLAST_FURNACE),
        group: Group::Production,
        count: 1,
        inputs: &[(b(STONE_BRICKS), 32), (b(CONCRETE), 8), (IRON_PLATE, 12), (MOTOR, 2)],
        blurb: "Makes steel from 2 iron ore, a coal and a quicklime, and slag on the side. It is 2×2×3 (R turns it \
                before you place it). Belts bring the three in at the hatches on its back and left; steel leaves \
                by the front, slag by the right hatch: if slag has nowhere to go it stops. No power.",
    },
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
    Recipe {
        output: b(QUARRY),
        group: Group::Production,
        count: 1,
        inputs: &[(IRON_PLATE, 12), (IRON_ROD, 8), (SCREW, 16), (COPPER_WIRE, 8)],
        blurb: "Digs the ground in front of it for real, about 2 blocks a second, into a belt or box beside it. \
                Leaves ore standing for miners. Hold it to see its box, R turns it; right-click it to choose the \
                size and depth. Needs 10 kW.",
    },
    BELT_MK2_RECIPE,
    BELT_MK3_RECIPE,
    BELT_MK4_RECIPE,
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
    STONE_PICKAXE_RECIPE,
    STONE_AXE_RECIPE,
    STONE_SHOVEL_RECIPE,
    IRON_PICKAXE_RECIPE,
    IRON_AXE_RECIPE,
    IRON_SHOVEL_RECIPE,
    STEEL_PICKAXE_RECIPE,
    STEEL_AXE_RECIPE,
    STEEL_SHOVEL_RECIPE,
    Recipe {
        output: SCANNER,
        group: Group::Tools,
        count: 1,
        inputs: &[(IRON_PLATE, 4), (COPPER_WIRE, 6), (SCREW, 4)],
        blurb: "Hold it and right-click to list the ore deposits within 48 blocks: what, how deep, which way \
                and how big. Never wears out.",
    },
    Recipe {
        output: CORE_DRILL,
        group: Group::Tools,
        count: 1,
        inputs: &[(IRON_PLATE, 6), (COPPER_WIRE, 2), (SCREW, 8)],
        blurb: "Hold right-click on the ground for 3 seconds to learn exactly how much ore lies beneath it, \
                and at what depth. Never wears out.",
    },
    Recipe {
        output: b(LAMP),
        group: Group::Building,
        count: 2,
        inputs: &[(b(GLASS), 1), (IRON_PLATE, 1), (COPPER_WIRE, 2)],
        blurb: "A block that lights up everything within about 20 blocks, day and night: caves, tunnels \
                and your factory after dark.",
    },
    Recipe {
        output: b(TORCH),
        group: Group::Building,
        count: 4,
        inputs: &[(STICK, 1), (b(COAL_ORE), 1)],
        blurb: "A small light for the first nights and caves: bright up close, about 5 blocks around. Stands \
                on top of any solid block.",
    },
    Recipe {
        output: b(PLANKS),
        group: Group::Materials,
        count: 4,
        inputs: &[(b(LOG), 1)],
        blurb: "Sawn from a log. Build with them, or make sticks, boxes and more from them. They burn, briefly.",
    },
    Recipe {
        output: STICK,
        group: Group::Materials,
        count: 4,
        inputs: &[(b(PLANKS), 2)],
        blurb: "Handles for tools, torches and ladders, and posts for power poles.",
    },
    Recipe {
        output: b(LADDER),
        group: Group::Building,
        count: 3,
        inputs: &[(STICK, 4)],
        blurb: "A frame you climb: stand in it and hold jump to go up, crouch to go down; let go and you stay \
                put. Stack them up a cliff or down a shaft; at the top, walk off onto the ledge.",
    },
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
    SPRING_BOOTS_RECIPE,
    SERVO_BOOTS_RECIPE,
    EXO_FRAME_RECIPE,
    MINING_RIG_RECIPE,
];

/// The item that is block `id`, to keep the tables short.
const fn b(id: BlockId) -> ItemId {
    ItemId::block(id)
}

#[cfg(test)]
mod tests;
