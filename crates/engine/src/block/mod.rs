//! Block registry: ids, one `BlockDef` row per block in `DEFS`, texture layers (`tex`), sound
//! materials (`sound`), and flat lookup tables (`OPAQUE`, `SOLID`, `FACE_TEX`…) for hot loops.
//! Every block is also the item with the same id (`item.rs`); blocks that should never be put back
//! into the world (ore, bedrock) are simply not placeable.
//!
//! To add a block: append an id constant (never renumber; ids will be saved) and bump
//! `BLOCK_COUNT`, add its `DEFS` row (`cube`, `ore` or `machine`), and any new texture layers in
//! `tex` with their patterns in `textures::pixel`. A machine also needs factory code (docs/CODEMAP.md).

pub type BlockId = u8;

pub const AIR: BlockId = 0;
pub const STONE: BlockId = 1;
pub const DIRT: BlockId = 2;
pub const GRASS: BlockId = 3;
pub const SAND: BlockId = 4;
pub const LOG: BlockId = 5;
pub const LEAVES: BlockId = 6;
pub const COAL_ORE: BlockId = 7;
pub const IRON_ORE: BlockId = 8;
pub const COPPER_ORE: BlockId = 9;
pub const BEDROCK: BlockId = 10;
/// What an ore block turns into once a miner has drawn its share out of the deposit.
pub const SPENT_ROCK: BlockId = 11;
pub const BELT: BlockId = 12;
pub const MINER: BlockId = 13;
pub const STORAGE: BlockId = 14;
pub const SMELTER: BlockId = 15;
pub const CONSTRUCTOR: BlockId = 16;
pub const SPLITTER: BlockId = 17;
pub const FILTER: BlockId = 18;
pub const RAMP_UP: BlockId = 19;
pub const RAMP_DOWN: BlockId = 20;
pub const LIFT: BlockId = 21;
pub const UNDERPASS_IN: BlockId = 22;
pub const UNDERPASS_OUT: BlockId = 23;
pub const GENERATOR: BlockId = 24;
pub const POLE: BlockId = 25;
pub const LAB: BlockId = 26;
pub const MINER_MK2: BlockId = 27;
pub const FAST_BELT: BlockId = 28;
/// Dropped now and then by leaves; grows into a tree on dirt or grass (`sim/saplings.rs`).
pub const SAPLING: BlockId = 29;
/// Rocks of generator version 2's provinces (`worldgen/`); building blocks that drop themselves.
pub const GRANITE: BlockId = 30;
pub const SANDSTONE: BlockId = 31;
pub const BASALT: BlockId = 32;
/// Deposit ores of generator version 2.
pub const LIMESTONE: BlockId = 33;
pub const QUARTZ_ORE: BlockId = 34;
/// Smelted from quartz; see-through (cutout).
pub const GLASS: BlockId = 35;
/// Surface hints above version 2's veins and lodes (`worldgen/geology.rs`): grass a shade off, over
/// stained soil; they drop dirt.
pub const RUSTY_SOIL: BlockId = 36;
pub const DARK_SOIL: BlockId = 37;
pub const GREEN_SOIL: BlockId = 38;
pub const PALE_SOIL: BlockId = 39;
/// The same hints where the surface is sand; they drop sand.
pub const RUSTY_SAND: BlockId = 40;
pub const DARK_SAND: BlockId = 41;
pub const GREEN_SAND: BlockId = 42;
pub const PALE_SAND: BlockId = 43;
/// Lights a wide area (`light::LAMP_LIGHT`); unpowered for now.
pub const LAMP: BlockId = 44;
/// Generator version 3's seas and ponds (`worldgen/water.rs`); not solid, not targetable, and blocks
/// placed into it replace it.
pub const WATER: BlockId = 45;
/// Flowing water (`sim/water.rs`), level 1 (thinnest) to 7; falling water is `FLOW_7`. Like `WATER`
/// otherwise. `flow(level)` and `flow_level` convert.
pub const FLOW_1: BlockId = 46;
pub const FLOW_7: BlockId = 52;
/// Water handling (`factory/pipes.rs`): a pump lifts sources out of the water it touches, pipes join
/// pumps to outlets, an outlet pours the water back out in front of it.
pub const PUMP: BlockId = 53;
pub const PIPE: BlockId = 54;
pub const OUTLET: BlockId = 55;
/// Digs the ground in a box in front of it, leaving a pit (`factory/quarry.rs`).
pub const QUARRY: BlockId = 56;
/// A small light on the block below it (`light::TORCH_LIGHT`: bright but short); drops when that block goes
/// (`sim/torches.rs`).
pub const TORCH: BlockId = 57;
/// Sawn from logs; a building block.
pub const PLANKS: BlockId = 58;
/// A see-through frame of rails and rungs: not solid; a body inside it climbs (`player.rs`).
pub const LADDER: BlockId = 59;
/// Fired from stone in a smelter (Masonry); a building block and later furnaces' lining.
pub const STONE_BRICKS: BlockId = 60;
/// The assembler's anchor cell (a 2×2×2 processor: `factory/process/specs.rs`).
pub const ASSEMBLER: BlockId = 61;
/// Every cell of a multi-block machine but its anchor (`factory/footprint/`): solid, drawn by the
/// machine's model; breaking it breaks the machine.
pub const MACHINE_PART: BlockId = 62;
/// Cast in an assembler from quicklime, sand and stone; a building block.
pub const CONCRETE: BlockId = 63;
/// The blast furnace's anchor cell (a 2×2×3 processor: steel from ore, coal and quicklime).
pub const BLAST_FURNACE: BlockId = 64;
/// The blast furnace's byproduct; a building block and fill for sites.
pub const SLAG: BlockId = 65;
/// Heavy industry (Steam Power, Ore Crushing, Bulk Storage), all processor rows (`factory/process/`):
/// the boiler (2×2×2), the steam turbine (3×2×2), the crusher (1×1×1) and the silo (2×2×3).
pub const BOILER: BlockId = 66;
pub const TURBINE: BlockId = 67;
pub const CRUSHER: BlockId = 68;
pub const SILO: BlockId = 69;
/// Electronics (Milestone 7): the arc furnace (2×2×2 processor: silicon from quartz and coal).
pub const ARC_FURNACE: BlockId = 70;
/// A cable hung down shafts and along tunnels (`factory/pole.rs`, `power_tools.rs`): a pole-kind machine.
pub const CABLE: BlockId = 71;
/// Solar power (Milestone 7): the solar panel (2×2×1) and the accumulator (2×2×2), processor rows (`factory/process/solar.rs`).
pub const SOLAR_PANEL: BlockId = 72;
pub const ACCUMULATOR: BlockId = 73;
/// Logic (`factory/sensor.rs`): watches the box or belt behind it, switches the machine in front.
pub const SENSOR: BlockId = 74;
/// A landing pad that keeps and launches construction drones (`factory/process/hangar.rs`, `drones/`).
pub const DRONE_PORT: BlockId = 75;
/// Bauxite (Milestone 9): the aluminium ore, found only in far deserts and basalt fields (generator version 5).
pub const BAUXITE_ORE: BlockId = 76;
/// The electrolytic cell (3×2×2 processor, `factory/process/specs.rs`): aluminium from crushed bauxite and quicklime.
pub const ELECTROLYTIC_CELL: BlockId = 77;
/// Track for trains (`factory/rail.rs`): a thin, non-solid machine that joins the rails beside it.
pub const RAIL: BlockId = 78;
/// The railway's docks (2×2×1 processors, `factory/process/docks.rs`): trains stop at them to load and unload.
pub const LOADING_DOCK: BlockId = 79;
pub const UNLOADING_DOCK: BlockId = 80;
/// Chemistry's ground (Milestone 10, generator version 6): oil sand (a deposit a pumpjack draws crude from) and uranium ore.
pub const OIL_SAND: BlockId = 81;
pub const URANIUM_ORE: BlockId = 82;
/// The chemistry machines, the washer, the research center, the water wheel the hoist, the centrifuge and the reactor, ids 83–96 (`chemistry.rs`).
mod chemistry;
pub use chemistry::*;
/// The recycler (the Recycling tech, `factory/process/recycler.rs`): a 2×2×2 machine that destroys items for coins.
pub const RECYCLER: BlockId = 97;
pub const BLOCK_COUNT: usize = 98;

mod make;
use make::{all, cube, frame, liquid, machine, ore, pillar};

/// Texture array layers (`block/tex.rs`).
pub mod tex;

/// Face order used everywhere (mesher, shaders, textures): +X, -X, +Y, -Y, +Z, -Z.
pub const FACE_TOP: usize = 2;
pub const FACE_BOTTOM: usize = 3;
pub const FACE_SIDE: usize = 0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Render {
    /// Not drawn by the chunk mesher (air, or machines the host draws as instanced models).
    None,
    /// Full cube that hides the faces of its neighbours.
    Opaque,
    /// Alpha-tested cube (leaves). Drawn in a separate pass so the opaque shader never uses `discard`.
    Cutout,
    /// Two crossed, alpha-tested quads (saplings), drawn with the cutout pass; hides no neighbour faces.
    Plant,
    /// Water: see-through, never targeted, replaced by whatever is placed into it.
    Liquid,
}

pub struct BlockDef {
    pub name: &'static str,
    pub render: Render,
    pub solid: bool,
    /// Seconds to break by hand; negative means unbreakable.
    pub break_time: f32,
    /// Texture layer per face (+X, -X, +Y, -Y, +Z, -Z).
    pub faces: [u16; 6],
    /// Item dropped when broken (the block item with this id).
    pub drop: BlockId,
    /// Sound material used for digging, breaking, placing and footsteps (see [`sound`]).
    pub sound: u8,
    /// Whether the item can be put back into the world.
    pub placeable: bool,
    /// The kind of block light it gives off: a row of `light::SOURCES` (0 for none).
    pub light: u8,
}

/// Sound materials. Must match `MATERIALS` in `web/src/audio/settings.ts`.
pub mod sound {
    pub const STONE: u8 = 0;
    pub const DIRT: u8 = 1;
    pub const GRASS: u8 = 2;
    pub const SAND: u8 = 3;
    pub const WOOD: u8 = 4;
    pub const LEAVES: u8 = 5;
    pub const METAL: u8 = 6;
}

pub(crate) const DEFS: [BlockDef; BLOCK_COUNT] = [
    BlockDef {
        name: "Air",
        render: Render::None,
        solid: false,
        break_time: 0.0,
        faces: all(0),
        drop: AIR,
        sound: sound::STONE,
        placeable: false,
        light: 0,
    },
    cube("Stone", 1.1, all(tex::STONE), STONE, sound::STONE),
    cube("Dirt", 0.45, all(tex::DIRT), DIRT, sound::DIRT),
    cube("Grass", 0.5, pillar(tex::GRASS_SIDE, tex::GRASS_TOP, tex::DIRT), DIRT, sound::GRASS),
    cube("Sand", 0.45, all(tex::SAND), SAND, sound::SAND),
    cube("Log", 0.9, pillar(tex::LOG_SIDE, tex::LOG_TOP, tex::LOG_TOP), LOG, sound::WOOD),
    BlockDef {
        name: "Leaves",
        render: Render::Cutout,
        solid: true,
        break_time: 0.2,
        faces: all(tex::LEAVES),
        drop: LEAVES,
        sound: sound::LEAVES,
        placeable: true,
        light: 0,
    },
    ore("Coal Ore", all(tex::COAL_ORE), COAL_ORE),
    ore("Iron Ore", all(tex::IRON_ORE), IRON_ORE),
    ore("Copper Ore", all(tex::COPPER_ORE), COPPER_ORE),
    BlockDef { placeable: false, ..cube("Bedrock", -1.0, all(tex::BEDROCK), BEDROCK, sound::STONE) },
    cube("Spent Rock", 0.7, all(tex::SPENT_ROCK), SPENT_ROCK, sound::STONE),
    machine("Conveyor Belt", false, 0.3, pillar(tex::FRAME, tex::BELT_TOP, tex::FRAME), BELT),
    machine("Miner Mk1", true, 0.8, pillar(tex::MINER_SIDE, tex::MINER_TOP, tex::FRAME), MINER),
    cube("Storage Box", 0.7, pillar(tex::BOX_SIDE, tex::BOX_TOP, tex::BOX_TOP), STORAGE, sound::WOOD),
    BlockDef {
        sound: sound::STONE,
        ..machine("Smelter", true, 1.0, pillar(tex::SMELTER_SIDE, tex::SMELTER_TOP, tex::SMELTER_TOP), SMELTER)
    },
    machine("Constructor", true, 0.8, pillar(tex::CONSTRUCTOR_SIDE, tex::CONSTRUCTOR_TOP, tex::FRAME), CONSTRUCTOR),
    machine("Splitter", false, 0.4, pillar(tex::FRAME, tex::SPLITTER_TOP, tex::FRAME), SPLITTER),
    machine("Filter", false, 0.4, pillar(tex::FRAME, tex::FILTER_TOP, tex::FRAME), FILTER),
    // Ramps from older worlds: plain belts now (slopes come from placement), so they drop a belt.
    BlockDef {
        drop: BELT,
        ..machine("Belt Ramp Up", false, 0.3, pillar(tex::RAMP_UP_SIDE, tex::BELT_TOP, tex::FRAME), RAMP_UP)
    },
    BlockDef {
        drop: BELT,
        ..machine("Belt Ramp Down", false, 0.3, pillar(tex::RAMP_DOWN_SIDE, tex::BELT_TOP, tex::FRAME), RAMP_DOWN)
    },
    machine("Belt Lift", false, 0.4, pillar(tex::LIFT_SIDE, tex::FRAME, tex::FRAME), LIFT),
    machine("Underpass", false, 0.4, pillar(tex::UNDERPASS_IN_SIDE, tex::BELT_TOP, tex::FRAME), UNDERPASS_IN),
    // The old separate exit: worlds from before one underpass piece hold it; it drops the new piece.
    BlockDef {
        placeable: false,
        drop: UNDERPASS_IN,
        ..machine(
            "Underpass Exit",
            false,
            0.4,
            pillar(tex::UNDERPASS_OUT_SIDE, tex::BELT_TOP, tex::FRAME),
            UNDERPASS_OUT,
        )
    },
    machine("Coal Generator", true, 0.8, pillar(tex::GENERATOR_SIDE, tex::GENERATOR_TOP, tex::FRAME), GENERATOR),
    machine("Power Pole", false, 0.3, pillar(tex::POLE_SIDE, tex::FRAME, tex::FRAME), POLE),
    machine("Research Lab", true, 0.8, pillar(tex::LAB_SIDE, tex::LAB_TOP, tex::FRAME), LAB),
    machine("Miner Mk2", true, 0.8, pillar(tex::MINER_MK2_SIDE, tex::MINER_TOP, tex::FRAME), MINER_MK2),
    machine("Belt Mk2", false, 0.3, pillar(tex::FRAME, tex::FAST_BELT_TOP, tex::FRAME), FAST_BELT),
    BlockDef {
        name: "Sapling",
        render: Render::Plant,
        solid: false,
        break_time: 0.1,
        faces: all(tex::SAPLING),
        drop: SAPLING,
        sound: sound::LEAVES,
        placeable: true,
        light: 0,
    },
    cube("Granite", 1.5, all(tex::GRANITE), GRANITE, sound::STONE),
    cube("Sandstone", 0.8, all(tex::SANDSTONE), SANDSTONE, sound::STONE),
    cube("Basalt", 1.6, all(tex::BASALT), BASALT, sound::STONE),
    ore("Limestone", all(tex::LIMESTONE), LIMESTONE),
    ore("Quartz Ore", all(tex::QUARTZ_ORE), QUARTZ_ORE),
    BlockDef { render: Render::Cutout, ..cube("Glass", 0.3, all(tex::GLASS), GLASS, sound::STONE) },
    cube("Rusty Soil", 0.5, pillar(tex::RUSTY_GRASS_SIDE, tex::RUSTY_GRASS_TOP, tex::RUSTY_SOIL), DIRT, sound::GRASS),
    cube("Dark Soil", 0.5, pillar(tex::DARK_GRASS_SIDE, tex::DARK_GRASS_TOP, tex::DARK_SOIL), DIRT, sound::GRASS),
    cube(
        "Verdigris Soil",
        0.5,
        pillar(tex::GREEN_GRASS_SIDE, tex::GREEN_GRASS_TOP, tex::GREEN_SOIL),
        DIRT,
        sound::GRASS,
    ),
    cube("Pale Soil", 0.5, pillar(tex::PALE_GRASS_SIDE, tex::PALE_GRASS_TOP, tex::PALE_SOIL), DIRT, sound::GRASS),
    cube("Rusty Sand", 0.45, all(tex::RUSTY_SAND), SAND, sound::SAND),
    cube("Dark Sand", 0.45, all(tex::DARK_SAND), SAND, sound::SAND),
    cube("Verdigris Sand", 0.45, all(tex::GREEN_SAND), SAND, sound::SAND),
    cube("Pale Sand", 0.45, all(tex::PALE_SAND), SAND, sound::SAND),
    BlockDef { light: crate::light::LAMP_LIGHT, ..cube("Lamp", 0.4, all(tex::LAMP), LAMP, sound::METAL) },
    liquid("Water"),
    liquid("Flowing Water"),
    liquid("Flowing Water"),
    liquid("Flowing Water"),
    liquid("Flowing Water"),
    liquid("Flowing Water"),
    liquid("Flowing Water"),
    liquid("Flowing Water"),
    machine("Pump", true, 0.8, pillar(tex::GENERATOR_SIDE, tex::STEEL, tex::FRAME), PUMP),
    machine("Pipe", true, 0.3, all(tex::STEEL), PIPE),
    machine("Outlet", true, 0.5, pillar(tex::STEEL, tex::FRAME, tex::FRAME), OUTLET),
    machine("Quarry", true, 1.0, pillar(tex::MINER_MK2_SIDE, tex::STEEL, tex::FRAME), QUARRY),
    BlockDef {
        name: "Torch",
        render: Render::Plant,
        solid: false,
        break_time: 0.05,
        faces: all(tex::TORCH),
        drop: TORCH,
        sound: sound::WOOD,
        placeable: true,
        light: crate::light::TORCH_LIGHT,
    },
    cube("Planks", 0.7, all(tex::PLANKS), PLANKS, sound::WOOD),
    frame("Ladder", 0.4, pillar(tex::LADDER, tex::LADDER_TOP, tex::LADDER_TOP), LADDER, sound::WOOD),
    cube("Stone Bricks", 1.3, all(tex::STONE_BRICKS), STONE_BRICKS, sound::STONE),
    machine("Assembler", true, 1.2, pillar(tex::ASSEMBLER_SIDE, tex::ASSEMBLER_TOP, tex::FRAME), ASSEMBLER),
    BlockDef { drop: AIR, placeable: false, ..machine("Machine Part", true, 1.2, all(tex::FRAME), MACHINE_PART) },
    cube("Concrete", 1.4, all(tex::CONCRETE), CONCRETE, sound::STONE),
    machine("Blast Furnace", true, 1.4, pillar(tex::BLAST_SIDE, tex::BLAST_TOP, tex::FRAME), BLAST_FURNACE),
    cube("Slag", 1.1, all(tex::SLAG), SLAG, sound::STONE),
    machine("Boiler", true, 1.4, pillar(tex::BOILER_SIDE, tex::BOILER_TOP, tex::FRAME), BOILER),
    machine("Steam Turbine", true, 1.4, pillar(tex::TURBINE_SIDE, tex::TURBINE_TOP, tex::FRAME), TURBINE),
    machine("Crusher", true, 1.3, pillar(tex::CRUSHER_SIDE, tex::CRUSHER_TOP, tex::FRAME), CRUSHER),
    machine("Silo", true, 1.4, pillar(tex::SILO_SIDE, tex::SILO_TOP, tex::FRAME), SILO),
    machine("Arc Furnace", true, 1.4, pillar(tex::ARC_SIDE, tex::ARC_TOP, tex::FRAME), ARC_FURNACE),
    machine("Power Cable", false, 0.1, all(tex::COPPER_WIRE), CABLE),
    machine("Solar Panel", true, 1.0, pillar(tex::SOLAR_SIDE, tex::SOLAR_TOP, tex::FRAME), SOLAR_PANEL),
    machine("Accumulator", true, 1.4, pillar(tex::ACCUMULATOR_SIDE, tex::ACCUMULATOR_TOP, tex::FRAME), ACCUMULATOR),
    machine("Sensor", true, 0.8, pillar(tex::FRAME, tex::CIRCUIT, tex::FRAME), SENSOR),
    machine("Drone Port", true, 1.4, pillar(tex::DRONE_PORT_SIDE, tex::DRONE_PORT_TOP, tex::FRAME), DRONE_PORT),
    ore("Bauxite Ore", all(tex::BAUXITE_ORE), BAUXITE_ORE),
    machine("Electrolytic Cell", true, 1.4, pillar(tex::CELL_SIDE, tex::CELL_TOP, tex::FRAME), ELECTROLYTIC_CELL),
    machine("Rail", false, 0.3, all(tex::RAIL), RAIL),
    machine("Loading Dock", true, 1.4, pillar(tex::DOCK_SIDE, tex::DOCK_LOAD_TOP, tex::FRAME), LOADING_DOCK),
    machine("Unloading Dock", true, 1.4, pillar(tex::DOCK_SIDE, tex::DOCK_UNLOAD_TOP, tex::FRAME), UNLOADING_DOCK),
    ore("Oil Sand", all(tex::OIL_SAND), OIL_SAND),
    ore("Uranium Ore", all(tex::URANIUM_ORE), URANIUM_ORE),
    machine("Pumpjack", true, 1.6, pillar(tex::PUMPJACK_SIDE, tex::PUMPJACK_TOP, tex::FRAME), PUMPJACK),
    machine("Refinery", true, 2.0, pillar(tex::REFINERY_SIDE, tex::REFINERY_TOP, tex::FRAME), REFINERY),
    machine("Cracker", true, 1.8, pillar(tex::CRACKER_SIDE, tex::CRACKER_TOP, tex::FRAME), CRACKER),
    machine("Chemical Plant", true, 1.8, pillar(tex::CHEM_SIDE, tex::CHEM_TOP, tex::FRAME), CHEMICAL_PLANT),
    machine("Diesel Generator", true, 1.8, pillar(tex::DIESEL_SIDE, tex::DIESEL_TOP, tex::FRAME), DIESEL_GENERATOR),
    machine("Electrolyser", true, 1.8, pillar(tex::ELECTROLYSER_SIDE, tex::ELECTROLYSER_TOP, tex::FRAME), ELECTROLYSER),
    machine("Ore Washer", true, 1.4, pillar(tex::WASHER_SIDE, tex::WASHER_TOP, tex::FRAME), WASHER),
    cube("Tailings", 0.9, all(tex::TAILINGS), TAILINGS, sound::STONE),
    machine("Research Center", true, 1.4, pillar(tex::LAB_SIDE, tex::LAB_TOP, tex::FRAME), RESEARCH_CENTER),
    machine("Water Wheel", true, 1.2, pillar(tex::PLANKS, tex::LOG_TOP, tex::FRAME), WATER_WHEEL),
    frame("Hoist Shaft", 0.6, pillar(tex::HOIST, tex::HOIST_TOP, tex::HOIST_TOP), HOIST, sound::METAL),
    machine("Hoist Winch", true, 1.4, pillar(tex::WINCH_SIDE, tex::FRAME, tex::FRAME), WINCH),
    machine("Centrifuge", true, 1.6, pillar(tex::CENTRIFUGE_SIDE, tex::CENTRIFUGE_TOP, tex::FRAME), CENTRIFUGE),
    machine("Nuclear Reactor", true, 2.4, pillar(tex::REACTOR_SIDE, tex::REACTOR_TOP, tex::FRAME), REACTOR),
    machine("Recycler", true, 1.6, pillar(tex::RECYCLER_SIDE, tex::RECYCLER_TOP, tex::FRAME), RECYCLER),
];

pub static BLOCK_DEFS: [BlockDef; BLOCK_COUNT] = DEFS;

/// Flat lookup tables for hot loops (`tables.rs`).
mod tables;
pub use tables::{ALT_TEX, CUTOUT, FACE_TEX, LIQUID, MESHED, OPAQUE, PLANT, SOLID};

/// Whether placing a block may take this cell (air or water), and so whether aiming passes through it.
#[inline]
pub fn replaceable(id: BlockId) -> bool {
    id == AIR || LIQUID[id as usize]
}

/// The flowing water block of `level` (1..=7).
#[inline]
pub fn flow(level: u8) -> BlockId {
    FLOW_1 + level.clamp(1, 7) - 1
}

/// The level of a flowing water block (1..=7), or `None` for any other block (sources included).
#[inline]
pub fn flow_level(id: BlockId) -> Option<u8> {
    (FLOW_1..=FLOW_7).contains(&id).then(|| id - FLOW_1 + 1)
}

#[inline]
pub fn def(id: BlockId) -> &'static BlockDef {
    BLOCK_DEFS.get(id as usize).unwrap_or(&BLOCK_DEFS[AIR as usize])
}

/// Which blocks are ores and what their deposits are called (ores.rs).
mod ores;
pub use ores::{is_ore, ore_label};
