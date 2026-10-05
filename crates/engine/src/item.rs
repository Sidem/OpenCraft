//! Item registry: `ItemId` and one `ItemDef` per item (name, stack size, look, the block it places).
//!
//! Invariants: ids below 256 are the blocks with the same number, and their rows are derived from
//! `block::BLOCK_DEFS`, so block ids and old saves stay valid items. Non-block items start at 256
//! (`EXTRA`, in id order). `ItemId::NONE` (air) is "no item". Every item is drawn as a box with three
//! texture layers (top, side, bottom) and proportions `size`: blocks are full cubes. Manufactured
//! loose items and HUD icons additionally use the presentation-only assemblies in `item_models`.
//!
//! To add an item: an id constant (append, never renumber; ids are saved) and its `EXTRA` row, plus a
//! texture layer in `block::tex` with its pattern in `textures::pixel` if it needs a new look.

use crate::block::{self, tex, BlockId, AIR, BLOCK_COUNT, FACE_BOTTOM, FACE_SIDE, FACE_TOP};
use crate::tools::{Tier, DEVICE_TIER, IRON_TIER, STEEL_TIER, STONE_TIER};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct ItemId(pub u16);

impl ItemId {
    pub const NONE: ItemId = ItemId(AIR as u16);

    /// The item that is the block `b`.
    pub const fn block(b: BlockId) -> ItemId {
        ItemId(b as u16)
    }

    /// The block this item puts into the world, if it can be placed. Every tier of a tiered machine
    /// places its family's block (`factory/tiers.rs`).
    pub fn places(self) -> Option<BlockId> {
        if let Some((block, _)) = crate::factory::tiers::placed_by(self) {
            return Some(block);
        }
        def(self).map(|d| d.places).filter(|&b| b != AIR)
    }

    /// A real item (not `NONE`, and in the table).
    pub fn is_valid(self) -> bool {
        self != ItemId::NONE && def(self).is_some()
    }
}

impl From<BlockId> for ItemId {
    fn from(b: BlockId) -> ItemId {
        ItemId::block(b)
    }
}

pub const IRON_INGOT: ItemId = ItemId(256);
pub const COPPER_INGOT: ItemId = ItemId(257);
pub const IRON_PLATE: ItemId = ItemId(258);
pub const IRON_ROD: ItemId = ItemId(259);
pub const SCREW: ItemId = ItemId(260);
pub const COPPER_WIRE: ItemId = ItemId(261);
pub const RED_PACK: ItemId = ItemId(262);
pub const GREEN_PACK: ItemId = ItemId(263);
pub const STONE_PICKAXE: ItemId = ItemId(264);
pub const STONE_AXE: ItemId = ItemId(265);
pub const STONE_SHOVEL: ItemId = ItemId(266);
pub const IRON_PICKAXE: ItemId = ItemId(267);
pub const IRON_AXE: ItemId = ItemId(268);
pub const IRON_SHOVEL: ItemId = ItemId(269);
pub const SCANNER: ItemId = ItemId(270);
pub const CORE_DRILL: ItemId = ItemId(271);
pub const STICK: ItemId = ItemId(272);
pub const GEAR: ItemId = ItemId(273);
/// Upgrade kits (`factory/upgrades.rs`), one per tier from Mk2.
pub const GREEN_KIT: ItemId = ItemId(274);
pub const QUICKLIME: ItemId = ItemId(275);
/// Mk2 of the processors (`factory/tiers.rs`): they place the Mk1 block at tier 1.
pub const SMELTER_MK2: ItemId = ItemId(276);
pub const CONSTRUCTOR_MK2: ItemId = ItemId(277);
/// Assembled from a rod, gears and wire (Assembly); blue packs, kits and later machines use it.
pub const MOTOR: ItemId = ItemId(278);
/// Steelmaking: a blast furnace's product, pressed into plates and beams by a constructor.
pub const STEEL_INGOT: ItemId = ItemId(279);
pub const STEEL_PLATE: ItemId = ItemId(280);
pub const STEEL_BEAM: ItemId = ItemId(281);
pub const STEEL_PICKAXE: ItemId = ItemId(282);
pub const STEEL_AXE: ItemId = ItemId(283);
pub const STEEL_SHOVEL: ItemId = ItemId(284);
/// Blue science (Blue Science): made by assemblers only; labs hold a slot for it.
pub const BLUE_PACK: ItemId = ItemId(285);
/// The kit that raises machines to Mk3 (blue stripe).
pub const BLUE_KIT: ItemId = ItemId(286);
/// Mk3 of belts, miners, smelters and constructors, and Mk2 and Mk3 of the multi-block machines
/// (`factory/tiers.rs`): each places its family's block at that tier.
pub const BELT_MK3: ItemId = ItemId(287);
pub const MINER_MK3: ItemId = ItemId(288);
pub const SMELTER_MK3: ItemId = ItemId(289);
pub const CONSTRUCTOR_MK3: ItemId = ItemId(290);
pub const ASSEMBLER_MK2: ItemId = ItemId(291);
pub const ASSEMBLER_MK3: ItemId = ItemId(292);
pub const BLAST_FURNACE_MK2: ItemId = ItemId(293);
pub const BLAST_FURNACE_MK3: ItemId = ItemId(294);
/// Mk2 and Mk3 of poles, boxes, pumps, quarries and labs, and Mk2 of the generator (same rule).
pub const POLE_MK2: ItemId = ItemId(295);
pub const POLE_MK3: ItemId = ItemId(296);
pub const BOX_MK2: ItemId = ItemId(297);
pub const BOX_MK3: ItemId = ItemId(298);
pub const PUMP_MK2: ItemId = ItemId(299);
pub const PUMP_MK3: ItemId = ItemId(300);
pub const QUARRY_MK2: ItemId = ItemId(301);
pub const QUARRY_MK3: ItemId = ItemId(302);
pub const LAB_MK2: ItemId = ItemId(303);
pub const LAB_MK3: ItemId = ItemId(304);
pub const GENERATOR_MK2: ItemId = ItemId(305);
/// Ore a crusher makes (Ore Crushing): it smelts to an ingot a piece, so an ore gives one and a half.
pub const CRUSHED_IRON: ItemId = ItemId(306);
pub const CRUSHED_COPPER: ItemId = ItemId(307);
/// Electronics: silicon from an arc furnace, and the circuit an assembler makes from it.
pub const SILICON: ItemId = ItemId(308);
pub const CIRCUIT: ItemId = ItemId(309);
/// Violet science (Violet Science): made by assemblers only; labs hold a fourth slot for it. The violet kit raises
/// machines to Mk4 (`factory/upgrades.rs`).
pub const VIOLET_PACK: ItemId = ItemId(310);
pub const VIOLET_KIT: ItemId = ItemId(311);
/// Mk4 of belts, miners, processors, poles and labs (`factory/tiers.rs`): each places its family's block at tier 3.
pub const BELT_MK4: ItemId = ItemId(312);
pub const MINER_MK4: ItemId = ItemId(313);
pub const SMELTER_MK4: ItemId = ItemId(314);
pub const CONSTRUCTOR_MK4: ItemId = ItemId(315);
pub const ASSEMBLER_MK4: ItemId = ItemId(316);
pub const BLAST_FURNACE_MK4: ItemId = ItemId(317);
pub const POLE_MK4: ItemId = ItemId(318);
pub const LAB_MK4: ItemId = ItemId(319);
/// Advanced Scanning: a scanner that lists deposits twice as far (`prospect.rs`).
pub const SCANNER_MK2: ItemId = ItemId(320);
/// The drone chain (Processors to Construction Drones; assembler recipes only): each part feeds the next, the drone last.
pub const PROCESSOR: ItemId = ItemId(321);
pub const SERVO: ItemId = ItemId(322);
pub const ACTUATOR: ItemId = ItemId(323);
pub const DRONE_CELL: ItemId = ItemId(324);
pub const GUIDANCE_MODULE: ItemId = ItemId(325);
pub const DRONE: ItemId = ItemId(326);
/// Drone port tiers 2 to 4 (`factory/tiers.rs`): each places the port block at its tier.
pub const DRONE_PORT_MK2: ItemId = ItemId(327);
pub const DRONE_PORT_MK3: ItemId = ItemId(328);
pub const DRONE_PORT_MK4: ItemId = ItemId(329);
/// Personal helpers (`helpers/`): a jetpack that burns coal from the pack, and a drone that fetches from boxes.
pub const JETPACK: ItemId = ItemId(330);
pub const PERSONAL_DRONE: ItemId = ItemId(331);
/// Earthworks (Milestone 8): marks terraforming sites for drone ports to work (`site_hands.rs`).
pub const PLANNER: ItemId = ItemId(332);
/// Worn gear (`equipment.rs`): one item fits one equipment slot, and none stacks.
pub const HAULER_PACK: ItemId = ItemId(333);
pub const HAULER_PACK_MK2: ItemId = ItemId(334);
pub const SPRING_BOOTS: ItemId = ItemId(335);
pub const SERVO_BOOTS: ItemId = ItemId(336);
pub const EXO_FRAME: ItemId = ItemId(337);
pub const MINING_RIG: ItemId = ItemId(338);
/// Aluminium (Bauxite Processing): crushed bauxite, the cell's ingot, the plate pressed from it and the battery.
pub const CRUSHED_BAUXITE: ItemId = ItemId(339);
pub const ALUMINIUM_INGOT: ItemId = ItemId(340);
pub const ALUMINIUM_PLATE: ItemId = ItemId(341);
pub const BATTERY: ItemId = ItemId(342);
/// Trains (the Trains tech): a locomotive, put on a rail node (`factory/trains.rs`).
pub const LOCOMOTIVE: ItemId = ItemId(343);
/// A wagon (the Freight tech), coupled behind a locomotive: a box's worth of slots, loaded at docks.
pub const WAGON: ItemId = ItemId(344);
/// A rail signal (the Freight tech), put on a rail node: one train at a time in the track between signals.
pub const RAIL_SIGNAL: ItemId = ItemId(345);
/// The hover pack (the Hover Pack tech): kept in the pack, charged near a power pole (`helpers/`).
pub const HOVER_PACK: ItemId = ItemId(346);
/// A cargo drone (the Cargo Drones tech): kept in a drone port, it flies loads to another port (`drones/cargo.rs`).
pub const CARGO_DRONE: ItemId = ItemId(347);

/// Stack size of every item except tools (whose stack is their uses: tools.rs).
pub const MAX_STACK: u32 = 64;

pub struct ItemDef {
    pub name: &'static str,
    /// Most items one slot holds.
    pub stack: u32,
    /// Texture layers of its model and icon: top, side, bottom.
    pub tex: [u16; 3],
    /// Model proportions (x, y, z); 1.0 on every axis is a full cube.
    pub size: [f32; 3],
    /// The block it places, or `AIR`.
    pub places: BlockId,
}

/// Looks up an item; `None` for ids in neither table.
#[inline]
pub fn def(id: ItemId) -> Option<&'static ItemDef> {
    let i = id.0 as usize;
    if i < 256 {
        BLOCK_ITEMS.get(i)
    } else {
        EXTRA.get(i - 256)
    }
}

pub fn name(id: ItemId) -> &'static str {
    def(id).map_or("", |d| d.name)
}

/// Most of `id` one slot holds (unknown items: [`MAX_STACK`]).
#[inline]
pub fn stack_size(id: ItemId) -> u32 {
    def(id).map_or(MAX_STACK, |d| d.stack)
}

/// An ingot: a small bar of metal.
const fn ingot(name: &'static str, layer: u16) -> ItemDef {
    ItemDef { name, stack: MAX_STACK, tex: [layer; 3], size: [0.9, 0.45, 0.55], places: AIR }
}

/// A part: `layer` on every face, proportions `size`.
const fn part(name: &'static str, layer: u16, size: [f32; 3]) -> ItemDef {
    ItemDef { name, stack: MAX_STACK, tex: [layer; 3], size, places: AIR }
}

/// A tier of a machine block: that block's look (the icon adds the tier chip).
const fn machine(name: &'static str, tex: [u16; 3]) -> ItemDef {
    ItemDef { name, stack: MAX_STACK, tex, size: [1.0; 3], places: AIR }
}

/// A tool: a flat plate showing it, whose stack size is its uses.
const fn tool(name: &'static str, layer: u16, tier: &Tier) -> ItemDef {
    ItemDef { name, stack: tier.uses, tex: [layer; 3], size: [0.7, 0.9, 0.12], places: AIR }
}

const EXTRA: [ItemDef; 92] = [
    ingot("Iron Ingot", tex::IRON_INGOT),
    ingot("Copper Ingot", tex::COPPER_INGOT),
    part("Iron Plate", tex::IRON_PLATE, [0.85, 0.14, 0.85]),
    part("Iron Rod", tex::IRON_ROD, [1.0, 0.2, 0.2]),
    part("Screws", tex::SCREW, [0.35, 0.35, 0.35]),
    part("Copper Wire", tex::COPPER_WIRE, [0.6, 0.4, 0.6]),
    part("Red Science Pack", tex::RED_PACK, [0.4, 0.6, 0.4]),
    part("Green Science Pack", tex::GREEN_PACK, [0.4, 0.6, 0.4]),
    tool("Stone Pickaxe", tex::STONE_PICKAXE, &STONE_TIER),
    tool("Stone Axe", tex::STONE_AXE, &STONE_TIER),
    tool("Stone Shovel", tex::STONE_SHOVEL, &STONE_TIER),
    tool("Iron Pickaxe", tex::IRON_PICKAXE, &IRON_TIER),
    tool("Iron Axe", tex::IRON_AXE, &IRON_TIER),
    tool("Iron Shovel", tex::IRON_SHOVEL, &IRON_TIER),
    tool("Scanner", tex::SCANNER, &DEVICE_TIER),
    tool("Core Drill", tex::CORE_DRILL, &DEVICE_TIER),
    part("Stick", tex::STICK, [1.0, 0.14, 0.14]),
    part("Gear", tex::GEAR, [0.8, 0.2, 0.8]),
    part("Green Kit", tex::stripe(1), [0.6, 0.45, 0.6]),
    part("Quicklime", tex::QUICKLIME, [0.6, 0.4, 0.6]),
    machine("Smelter Mk2", [tex::SMELTER_TOP, tex::SMELTER_SIDE, tex::SMELTER_TOP]),
    machine("Constructor Mk2", [tex::CONSTRUCTOR_TOP, tex::CONSTRUCTOR_SIDE, tex::FRAME]),
    part("Motor", tex::MOTOR, [0.55, 0.55, 0.8]),
    ingot("Steel Ingot", tex::STEEL_INGOT),
    part("Steel Plate", tex::STEEL_PLATE, [0.85, 0.14, 0.85]),
    part("Steel Beam", tex::STEEL_BEAM, [1.0, 0.3, 0.3]),
    tool("Steel Pickaxe", tex::STEEL_PICKAXE, &STEEL_TIER),
    tool("Steel Axe", tex::STEEL_AXE, &STEEL_TIER),
    tool("Steel Shovel", tex::STEEL_SHOVEL, &STEEL_TIER),
    part("Blue Science Pack", tex::BLUE_PACK, [0.4, 0.6, 0.4]),
    part("Blue Kit", tex::stripe(2), [0.6, 0.45, 0.6]),
    machine("Belt Mk3", [tex::FRAME, tex::BELT_MK3_TOP, tex::FRAME]),
    machine("Miner Mk3", [tex::MINER_TOP, tex::MINER_MK3_SIDE, tex::FRAME]),
    machine("Smelter Mk3", [tex::SMELTER_TOP, tex::SMELTER_SIDE, tex::SMELTER_TOP]),
    machine("Constructor Mk3", [tex::CONSTRUCTOR_TOP, tex::CONSTRUCTOR_SIDE, tex::FRAME]),
    machine("Assembler Mk2", [tex::ASSEMBLER_TOP, tex::ASSEMBLER_SIDE, tex::FRAME]),
    machine("Assembler Mk3", [tex::ASSEMBLER_TOP, tex::ASSEMBLER_SIDE, tex::FRAME]),
    machine("Blast Furnace Mk2", [tex::BLAST_TOP, tex::BLAST_SIDE, tex::FRAME]),
    machine("Blast Furnace Mk3", [tex::BLAST_TOP, tex::BLAST_SIDE, tex::FRAME]),
    machine("Power Pole Mk2", [tex::FRAME, tex::POLE_SIDE, tex::FRAME]),
    machine("Power Pole Mk3", [tex::FRAME, tex::POLE_SIDE, tex::FRAME]),
    machine("Storage Box Mk2", [tex::BOX_TOP, tex::BOX_SIDE, tex::BOX_TOP]),
    machine("Storage Box Mk3", [tex::BOX_TOP, tex::BOX_SIDE, tex::BOX_TOP]),
    machine("Pump Mk2", [tex::STEEL, tex::GENERATOR_SIDE, tex::FRAME]),
    machine("Pump Mk3", [tex::STEEL, tex::GENERATOR_SIDE, tex::FRAME]),
    machine("Quarry Mk2", [tex::STEEL, tex::MINER_MK2_SIDE, tex::FRAME]),
    machine("Quarry Mk3", [tex::STEEL, tex::MINER_MK2_SIDE, tex::FRAME]),
    machine("Research Lab Mk2", [tex::LAB_TOP, tex::LAB_SIDE, tex::FRAME]),
    machine("Research Lab Mk3", [tex::LAB_TOP, tex::LAB_SIDE, tex::FRAME]),
    machine("Coal Generator Mk2", [tex::GENERATOR_TOP, tex::GENERATOR_SIDE, tex::FRAME]),
    part("Crushed Iron", tex::CRUSHED_IRON, [0.6, 0.4, 0.6]),
    part("Crushed Copper", tex::CRUSHED_COPPER, [0.6, 0.4, 0.6]),
    ingot("Silicon", tex::SILICON),
    part("Circuit", tex::CIRCUIT, [0.7, 0.1, 0.7]),
    part("Violet Science Pack", tex::VIOLET_PACK, [0.4, 0.6, 0.4]),
    part("Violet Kit", tex::stripe(3), [0.6, 0.45, 0.6]),
    machine("Belt Mk4", [tex::FRAME, tex::BELT_MK4_TOP, tex::FRAME]),
    machine("Miner Mk4", [tex::MINER_TOP, tex::MINER_MK4_SIDE, tex::FRAME]),
    machine("Smelter Mk4", [tex::SMELTER_TOP, tex::SMELTER_SIDE, tex::SMELTER_TOP]),
    machine("Constructor Mk4", [tex::CONSTRUCTOR_TOP, tex::CONSTRUCTOR_SIDE, tex::FRAME]),
    machine("Assembler Mk4", [tex::ASSEMBLER_TOP, tex::ASSEMBLER_SIDE, tex::FRAME]),
    machine("Blast Furnace Mk4", [tex::BLAST_TOP, tex::BLAST_SIDE, tex::FRAME]),
    machine("Power Pole Mk4", [tex::FRAME, tex::POLE_SIDE, tex::FRAME]),
    machine("Research Lab Mk4", [tex::LAB_TOP, tex::LAB_SIDE, tex::FRAME]),
    tool("Scanner Mk2", tex::SCANNER_MK2, &DEVICE_TIER),
    part("Processor", tex::PROCESSOR, [0.6, 0.12, 0.6]),
    part("Servo", tex::SERVO, [0.55, 0.4, 0.4]),
    part("Actuator", tex::ACTUATOR, [0.8, 0.4, 0.4]),
    part("Drone Cell", tex::DRONE_CELL, [0.5, 0.6, 0.35]),
    part("Guidance Module", tex::GUIDANCE, [0.55, 0.1, 0.55]),
    part("Drone", tex::DRONE, [0.8, 0.3, 0.8]),
    machine("Drone Port Mk2", [tex::DRONE_PORT_TOP, tex::DRONE_PORT_SIDE, tex::FRAME]),
    machine("Drone Port Mk3", [tex::DRONE_PORT_TOP, tex::DRONE_PORT_SIDE, tex::FRAME]),
    machine("Drone Port Mk4", [tex::DRONE_PORT_TOP, tex::DRONE_PORT_SIDE, tex::FRAME]),
    tool("Coal Jetpack", tex::JETPACK, &DEVICE_TIER),
    tool("Personal Drone", tex::PERSONAL_DRONE, &DEVICE_TIER),
    tool("Planner", tex::PLANNER, &DEVICE_TIER),
    tool("Hauler Pack", tex::HAULER_PACK, &DEVICE_TIER),
    tool("Hauler Pack Mk2", tex::HAULER_PACK_MK2, &DEVICE_TIER),
    tool("Spring Boots", tex::SPRING_BOOTS, &DEVICE_TIER),
    tool("Servo Boots", tex::SERVO_BOOTS, &DEVICE_TIER),
    tool("Exo Frame", tex::EXO_FRAME, &DEVICE_TIER),
    tool("Mining Rig", tex::MINING_RIG, &DEVICE_TIER),
    part("Crushed Bauxite", tex::CRUSHED_BAUXITE, [0.6, 0.4, 0.6]),
    ingot("Aluminium Ingot", tex::ALUMINIUM_INGOT),
    part("Aluminium Plate", tex::ALUMINIUM_PLATE, [0.85, 0.14, 0.85]),
    part("Battery", tex::BATTERY, [0.4, 0.6, 0.4]),
    ItemDef { name: "Locomotive", stack: 8, tex: [tex::LOCOMOTIVE; 3], size: [0.9, 0.7, 0.9], places: AIR },
    ItemDef { name: "Wagon", stack: 8, tex: [tex::WAGON; 3], size: [0.9, 0.6, 0.9], places: AIR },
    ItemDef { name: "Rail Signal", stack: 64, tex: [tex::SIGNAL; 3], size: [0.4, 0.8, 0.4], places: AIR },
    tool("Hover Pack", tex::HOVER_PACK, &DEVICE_TIER),
    part("Cargo Drone", tex::CARGO_DRONE, [0.8, 0.4, 0.8]),
];

/// One row per block: its name and faces, placeable blocks place themselves.
static BLOCK_ITEMS: [ItemDef; BLOCK_COUNT] = {
    const EMPTY: ItemDef = ItemDef { name: "", stack: MAX_STACK, tex: [0; 3], size: [1.0; 3], places: AIR };
    let mut t = [EMPTY; BLOCK_COUNT];
    let mut i = 0;
    while i < BLOCK_COUNT {
        let b = &block::DEFS[i];
        let f = b.faces;
        t[i] = ItemDef {
            name: b.name,
            stack: MAX_STACK,
            tex: [f[FACE_TOP], f[FACE_SIDE], f[FACE_BOTTOM]],
            size: [1.0; 3],
            places: if b.placeable { i as BlockId } else { AIR },
        };
        i += 1;
    }
    t
};

#[cfg(test)]
mod tests;
