//! Block registry: ids, the `BlockDef` row type (the rows: `DEFS` in `defs.rs`), texture layers (`tex`), sound
//! materials (`sound`), and flat lookup tables (`OPAQUE`, `SOLID`, `FACE_TEX`…) for hot loops.
//! Every block is also the item with the same id (`item.rs`); blocks that should never be put back
//! into the world (ore, bedrock) are simply not placeable.
//!
//! To add a block: append an id constant (never renumber; ids will be saved) and bump
//! `BASE_COUNT`, add its `BASE_DEFS` row in `defs.rs` (`cube`, `ore` or `machine`; decor blocks go in `decor.rs`), and
//! any new texture layers in `tex` with their patterns in `textures::pixel`. A machine also needs factory code
//! (docs/CODEMAP.md).

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
/// The chip fab (the Wafers tech, `factory/process/fab.rs`): a 4×4×3 clean room that makes wafers and AI accelerators.
pub const CHIP_FAB: BlockId = 98;
/// A fibre node: the data grid's post (factory/fibre.rs, Milestone 11).
pub const FIBRE_NODE: BlockId = 99;
/// The AI datacenter and the cooling tower (`factory/process/datacenter.rs`, `tower.rs`; Milestone 11).
pub const DATACENTER: BlockId = 100;
/// The cooling tower (`factory/process/tower.rs`, Milestone 11).
pub const COOLING_TOWER: BlockId = 101;
/// The AI lab (`factory/process/ailab.rs`).
pub const AI_LAB: BlockId = 102;
/// The optimizer node (`factory/process/optimizer.rs`).
pub const OPTIMIZER: BlockId = 103;
/// The laser emitter and receiver (`factory/laser.rs`, Milestone 11).
pub const LASER_EMITTER: BlockId = 104;
pub const LASER_RECEIVER: BlockId = 105;
/// The laser mirror (turns a beam 90°) and the data receiver (a beam that ends in it joins two data grids).
pub const LASER_MIRROR: BlockId = 106;
pub const DATA_RECEIVER: BlockId = 107;
/// The swarm hub (`factory/process/hub.rs`): drone ports near it keep more drones.
pub const SWARM_HUB: BlockId = 108;
/// The launch site (`factory/process/launch.rs`, Milestone 12): a 7×7 pad and gantry that builds rockets and launches them.
pub const LAUNCH_SITE: BlockId = 109;
/// The rectenna (`factory/process/rectenna.rs`): a 5×5 field that turns a power satellite's beam into power.
pub const RECTENNA: BlockId = 110;
/// The ground station (`factory/process/ground.rs`): a dish that downlinks science satellites' data as compute.
pub const GROUND_STATION: BlockId = 111;
/// The long-distance pole (`factory/pole.rs`): a 2×2×3 tower that spans far to other long-distance poles.
pub const LONG_POLE: BlockId = 112;
/// The pod launcher (`factory/process/pods.rs`, step 12.4): belts load it, a drop pod carries the cargo to a landing pad.
pub const POD_LAUNCHER: BlockId = 113;
/// The landing pad (`factory/process/pods.rs`): drop pods from launchers on its channel come down on it; belts empty it.
pub const LANDING_PAD: BlockId = 114;
/// The upgrader (`factory/process/upgrader.rs`): raises a Mk1 item to a chosen tier, using up the kits of each step.
pub const UPGRADER: BlockId = 115;
/// The town depot (`factory/process/depot.rs`, step 13.1): placed at a fitting site it founds a town (`settlements/`).
pub const TOWN_DEPOT: BlockId = 116;
/// The forest's blocks, ids 117–123 (`forest.rs`).
mod forest;
pub use forest::*;
/// The ruins' blocks, ids 124–127 (`ruins.rs`).
mod ruins;
pub use ruins::*;
/// The decor set, ids 129–142 (`decor.rs`).
mod decor;
pub use decor::*;
/// The coal miner, id 143 (`coal_miner.rs`).
mod coal_miner;
pub use coal_miner::*;
/// The house set, ids 144–156 (`house.rs`).
mod house;
pub use house::*;
/// The belt ejector, id 157 (`ejector.rs`).
mod ejector;
pub use ejector::*;
const BASE_COUNT: usize = 129;
pub const BLOCK_COUNT: usize = BASE_COUNT + DECOR_COUNT + COAL_MINER_COUNT + HOUSE_COUNT + EJECTOR_COUNT;

/// Every block's row (`defs.rs`), built with `make.rs`.
mod defs;
mod make;
pub(crate) use defs::DEFS;

/// Texture array layers (`block/tex.rs`).
pub mod tex;
pub mod turn;

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

#[derive(Clone, Copy)]
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
