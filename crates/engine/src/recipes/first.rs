//! Hand recipes of the first machines, tools and building blocks (miner, belts, smelter, power, labs, the first packs
//! and kits, assembler, blast furnace, quarry, scanner, lamps, wood), listed in `RECIPES` (`mod.rs`), whose order is
//! the build menu's. To add one: a `pub const` in the file of its era or set, and its name in `RECIPES`.

use crate::block::*;
use crate::item::{
    COPPER_INGOT, COPPER_WIRE, CORE_DRILL, GEAR, GREEN_KIT, GREEN_PACK, IRON_INGOT, IRON_PLATE, IRON_ROD, MOTOR,
    RED_PACK, SCANNER, SCREW, STICK, TELESCOPE,
};

use super::{b, Group, Recipe};

pub const MINER_RECIPE: Recipe = Recipe {
    output: b(MINER),
    group: Group::Production,
    count: 1,
    inputs: &[(IRON_PLATE, 5), (IRON_ROD, 4), (COPPER_WIRE, 6)],
    blurb: "Place it against an ore block. It drills the whole deposit, recovers 60% of what it draws, \
            and pushes ore into a belt, box, smelter or generator beside it. Needs power (5 kW).",
};

pub const BELT_RECIPE: Recipe = Recipe {
    output: b(BELT),
    group: Group::Logistics,
    count: 4,
    inputs: &[(IRON_PLATE, 1), (IRON_ROD, 1)],
    blurb: "Carries items. Hold right-click and drag to lay a line; it climbs and drops one-block steps \
            by itself. Belts feed belts, machines, and belts from the side. Press R on a belt to turn it.",
};

pub const STORAGE_RECIPE: Recipe = Recipe {
    output: b(STORAGE),
    group: Group::Logistics,
    count: 1,
    inputs: &[(b(PLANKS), 8), (IRON_PLATE, 2)],
    blurb: "Holds 24 stacks. Belts deliver into it; a belt leading away from it is fed from it. \
            Right-click to empty it into your inventory.",
};

pub const SMELTER_RECIPE: Recipe = Recipe {
    output: b(SMELTER),
    group: Group::Production,
    count: 1,
    inputs: &[(b(STONE), 16)],
    blurb: "A stone furnace. Melts iron or copper ore into ingots, sand into glass (and, with Masonry, stone into bricks) while \
            it has fuel: coal ore or logs. Belts bring both in; a belt leading away takes the ingots. \
            Right-click to open it.",
};

pub const CONSTRUCTOR_RECIPE: Recipe = Recipe {
    output: b(CONSTRUCTOR),
    group: Group::Production,
    count: 1,
    inputs: &[(IRON_INGOT, 10), (COPPER_INGOT, 4), (b(STONE), 8)],
    blurb: "Shapes ingots into parts: plates, rods, screws and wire. Right-click to choose what it makes; \
            belts bring the ingots in and take the parts away.",
};

pub const SPLITTER_RECIPE: Recipe = Recipe {
    output: b(SPLITTER),
    group: Group::Logistics,
    count: 1,
    inputs: &[(IRON_PLATE, 2), (b(BELT), 2)],
    blurb: "Takes items from belts leading into it and shares them between the belts leading away in \
            front, to the left and to the right.",
};

pub const FILTER_RECIPE: Recipe = Recipe {
    output: b(FILTER),
    group: Group::Logistics,
    count: 1,
    inputs: &[(IRON_PLATE, 2), (COPPER_WIRE, 2), (b(BELT), 2)],
    blurb: "Sends the item you choose straight on and everything else to the left and right. \
            Right-click to choose the item.",
};

pub const LIFT_RECIPE: Recipe = Recipe {
    output: b(LIFT),
    group: Group::Logistics,
    count: 2,
    inputs: &[(IRON_ROD, 2), (b(BELT), 2)],
    blurb: "Carries items straight up. Stack lifts facing the same way to climb higher; the top one \
            hands items on one block ahead and one up. You can climb a lift stack like a ladder.",
};

pub const UNDERPASS_RECIPE: Recipe = Recipe {
    output: b(UNDERPASS_IN),
    group: Group::Logistics,
    count: 2,
    inputs: &[(b(BELT), 4)],
    blurb: "Carries items under up to 4 blocks of anything. Made in pairs: place two facing the same way, the \
            first takes items in, the next one ahead brings them out. Drag a belt line over an obstacle and \
            it places them for you. It moves items at belt speed.",
};

pub const PUMP_RECIPE: Recipe = Recipe {
    output: b(PUMP),
    group: Group::Logistics,
    count: 1,
    inputs: &[(IRON_PLATE, 6), (IRON_ROD, 4), (COPPER_WIRE, 6)],
    blurb: "Lifts 2 blocks of still water a second out of the water it touches, highest first. Pipe it to \
            an outlet; needs 5 kW. It can't lower the sea.",
};

pub const PIPE_RECIPE: Recipe = Recipe {
    output: b(PIPE),
    group: Group::Logistics,
    count: 4,
    inputs: &[(IRON_PLATE, 2)],
    blurb: "Joins pumps to outlets. Pipes connect on every side.",
};

pub const OUTLET_RECIPE: Recipe = Recipe {
    output: b(OUTLET),
    group: Group::Logistics,
    count: 1,
    inputs: &[(IRON_PLATE, 3), (IRON_ROD, 2)],
    blurb: "Pours the water its pipes bring out in front of it (it faces the way you do), filling from the \
            bottom up.",
};

pub const GENERATOR_RECIPE: Recipe = Recipe {
    output: b(GENERATOR),
    group: Group::Power,
    count: 1,
    inputs: &[(IRON_PLATE, 4), (COPPER_WIRE, 8), (b(STONE), 12)],
    blurb: "Burns coal ore (270 kJ) or logs (135 kJ) into up to 60 kW, and only as much as its grid uses. \
            Place a power pole within 5 blocks; a belt or a miner beside it brings fuel. Right-click to open it.",
};

pub const POLE_RECIPE: Recipe = Recipe {
    output: b(POLE),
    group: Group::Power,
    count: 2,
    inputs: &[(STICK, 2), (COPPER_WIRE, 2)],
    blurb: "Links to every pole within 10 blocks and powers generators and machines within 5. \
            Miners, constructors, splitters, filters, labs, pumps and quarries need power.",
};

pub const LAB_RECIPE: Recipe = Recipe {
    output: b(LAB),
    group: Group::Science,
    count: 1,
    inputs: &[(IRON_PLATE, 6), (COPPER_WIRE, 8), (b(BELT), 4)],
    blurb: "Uses science packs to research new machines (press T to choose what). Belts bring packs in; \
            needs power.",
};

pub const RED_PACK_RECIPE: Recipe = Recipe {
    output: RED_PACK,
    group: Group::Science,
    count: 1,
    inputs: &[(IRON_PLATE, 1), (COPPER_WIRE, 2)],
    blurb: "A lab uses these to research the first techs.",
};

pub const GREEN_PACK_RECIPE: Recipe = Recipe {
    output: GREEN_PACK,
    group: Group::Science,
    count: 1,
    inputs: &[(b(BELT), 2), (SCREW, 4)],
    blurb: "A lab uses these, with red packs, for later techs.",
};

pub const GREEN_KIT_RECIPE: Recipe = Recipe {
    output: GREEN_KIT,
    group: Group::Materials,
    count: 4,
    inputs: &[(GEAR, 2), (SCREW, 4), (COPPER_WIRE, 2)],
    blurb: "Upgrades machines to Mk2 (green stripe) in place. Hold kits and right-click a miner, smelter or \
            constructor (4 kits), or hold the button and drag along belts (1 kit each).",
};

pub const ASSEMBLER_RECIPE: Recipe = Recipe {
    output: b(ASSEMBLER),
    group: Group::Production,
    count: 1,
    inputs: &[(IRON_PLATE, 12), (GEAR, 6), (COPPER_WIRE, 12), (IRON_ROD, 4)],
    blurb: "Puts parts together: motors, concrete, packs and kits. It is 2×2×2 (R turns it before you place \
            it). Belts bring parts in at the hatches on its back and sides; the front hatch gives. 20 kW.",
};

pub const BLAST_FURNACE_RECIPE: Recipe = Recipe {
    output: b(BLAST_FURNACE),
    group: Group::Production,
    count: 1,
    inputs: &[(b(STONE_BRICKS), 32), (b(CONCRETE), 8), (IRON_PLATE, 12), (MOTOR, 2)],
    blurb: "Makes steel from 2 iron ore, a coal and a quicklime, and slag on the side. It is 2×2×3 (R turns it \
            before you place it). Belts bring the three in at the hatches on its back and left; steel leaves \
            by the front, slag by the right hatch: if slag has nowhere to go it stops. No power.",
};

pub const QUARRY_RECIPE: Recipe = Recipe {
    output: b(QUARRY),
    group: Group::Production,
    count: 1,
    inputs: &[(IRON_PLATE, 12), (IRON_ROD, 8), (SCREW, 16), (COPPER_WIRE, 8)],
    blurb: "Digs the ground in front of it for real, about 2 blocks a second, into a belt or box beside it. \
            Leaves ore standing for miners. Hold it to see its box, R turns it; right-click it to choose the \
            size and depth. Needs 10 kW.",
};

pub const SCANNER_RECIPE: Recipe = Recipe {
    output: SCANNER,
    group: Group::Tools,
    count: 1,
    inputs: &[(IRON_PLATE, 4), (COPPER_WIRE, 6), (SCREW, 4)],
    blurb: "Hold it and right-click to list the ore deposits within 48 blocks: what, how deep, which way \
            and how big. Never wears out.",
};

pub const CORE_DRILL_RECIPE: Recipe = Recipe {
    output: CORE_DRILL,
    group: Group::Tools,
    count: 1,
    inputs: &[(IRON_PLATE, 6), (COPPER_WIRE, 2), (SCREW, 8)],
    blurb: "Hold right-click on the ground for 3 seconds to learn exactly how much ore lies beneath it, \
            and at what depth. Never wears out.",
};

pub const TELESCOPE_RECIPE: Recipe = Recipe {
    output: TELESCOPE,
    group: Group::Tools,
    count: 1,
    inputs: &[(b(GLASS), 2), (IRON_PLATE, 2), (IRON_ROD, 2)],
    blurb: "Hold it and right-click to look through it; the wheel zooms. Left-click pins the ground you look at, \
            and every pin shows as a column of light in its colour, however far. Never wears out.",
};

pub const LAMP_RECIPE: Recipe = Recipe {
    output: b(LAMP),
    group: Group::Building,
    count: 2,
    inputs: &[(b(GLASS), 1), (IRON_PLATE, 1), (COPPER_WIRE, 2)],
    blurb: "A block that lights up everything within about 20 blocks, day and night: caves, tunnels \
            and your factory after dark.",
};

pub const TORCH_RECIPE: Recipe = Recipe {
    output: b(TORCH),
    group: Group::Building,
    count: 4,
    inputs: &[(STICK, 1), (b(COAL_ORE), 1)],
    blurb: "A small light for the first nights and caves: bright up close, about 5 blocks around. Stands \
            on top of any solid block.",
};

pub const PLANKS_RECIPE: Recipe = Recipe {
    output: b(PLANKS),
    group: Group::Materials,
    count: 4,
    inputs: &[(b(LOG), 1)],
    blurb: "Sawn from a log. Build with them, or make sticks, boxes and more from them. They burn, briefly.",
};

pub const STICK_RECIPE: Recipe = Recipe {
    output: STICK,
    group: Group::Materials,
    count: 4,
    inputs: &[(b(PLANKS), 2)],
    blurb: "Handles for tools, torches and ladders, and posts for power poles.",
};

pub const LADDER_RECIPE: Recipe = Recipe {
    output: b(LADDER),
    group: Group::Building,
    count: 3,
    inputs: &[(STICK, 4)],
    blurb: "A frame you climb: stand in it and hold jump to go up, crouch to go down; let go and you stay \
            put. Stack them up a cliff or down a shaft; at the top, walk off onto the ledge.",
};
