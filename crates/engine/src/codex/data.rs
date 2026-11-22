//! Chemical element forms and facts for the Codex (`codex/mod.rs`): pure data mapping existing items
//! and blocks to `(atomic_number, Form)`. Adding a new chemistry family appends rows here; nothing
//! else in the engine needs to change.

use crate::block::{
    BAUXITE_ORE, COAL_ORE, CONCRETE, COPPER_ORE, GAS_SHALE, GLASS, IRON_ORE, LIMESTONE, OIL_SAND, QUARTZ_ORE,
    URANIUM_ORE,
};
use crate::item::{
    self, ItemId, ACID_BARREL, ALUMINIUM_INGOT, ALUMINIUM_PLATE, CHARCOAL, CIRCUIT, COPPER_INGOT, COPPER_WIRE,
    CRUDE_BARREL, CRUSHED_BAUXITE, CRUSHED_COPPER, CRUSHED_IRON, DIESEL_BARREL, ETHANE_BARREL, ETHYLENE_BARREL,
    FUEL_CELL, HEAVY_OIL_BARREL, HYDROGEN_BARREL, IRON_INGOT, IRON_PLATE, LIGHT_FRAME, LPG_BARREL, LUBRICANT_BARREL,
    METHANE_BARREL, NAPHTHA_BARREL, OXYGEN_BARREL, PIGMENT_BLACK, PIGMENT_BLUE, PIGMENT_GREEN, PIGMENT_RED,
    PIGMENT_WHITE, PIGMENT_YELLOW, PLASTIC, PROCESSOR, PURE_WATER_BARREL, QUICKLIME, RAW_GAS_BARREL,
    ROCKET_FUEL_BARREL, SCRAP_METAL, SILICON, STEEL_INGOT, STEEL_PLATE, SULFUR, WAFER, WASHED_BAUXITE, WASHED_COPPER,
    WASHED_IRON,
};

/// Chemical form of an item in the periodic table card.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Form {
    Ore = 0,
    Oxide = 1,
    Carbonate = 2,
    Pure = 3,
    Alloy = 4,
    Compound = 5,
    Gas = 6,
    Liquid = 7,
}

/// One in-game item that represents an element in a specific chemical form.
pub struct ElementForm {
    pub z: u8,
    pub item: ItemId,
    pub form: Form,
    pub note: &'static str,
}

/// Real-world fact and in-game role for an element present in this world.
pub struct ElementInfo {
    pub z: u8,
    pub fact: &'static str,
    pub role: &'static str,
}

const fn entry(z: u8, item: ItemId, form: Form, note: &'static str) -> ElementForm {
    ElementForm { z, item, form, note }
}

const fn block_entry(z: u8, block: crate::block::BlockId, form: Form, note: &'static str) -> ElementForm {
    ElementForm { z, item: ItemId::block(block), form, note }
}

/// Every item/block mapped to an element and chemical form (ordered by atomic number).
pub const FORMS: &[ElementForm] = &[
    // 1 — Hydrogen
    entry(1, HYDROGEN_BARREL, Form::Gas, "H₂ gas from water electrolysis or steam reforming"),
    entry(1, PURE_WATER_BARREL, Form::Liquid, "Ultra-pure H₂O for semiconductor rinsing"),
    entry(1, METHANE_BARREL, Form::Gas, "CH₄ — lightest alkane, main share of natural gas"),
    entry(1, ETHANE_BARREL, Form::Gas, "C₂H₆ — natural gas liquid cracked to ethylene"),
    entry(1, LPG_BARREL, Form::Gas, "C₃H₈ / C₄H₁₀ — liquefied petroleum gas"),
    entry(1, ETHYLENE_BARREL, Form::Gas, "C₂H₄ — unsaturated monomer for polyethylene"),
    entry(1, ROCKET_FUEL_BARREL, Form::Liquid, "High-energy H₂/O₂ propellant for orbital launches"),
    // 6 — Carbon
    block_entry(6, COAL_ORE, Form::Ore, "Fossil carbon — early fuel and blast-furnace reductant"),
    entry(6, CHARCOAL, Form::Pure, "Amorphous carbon left when wood pyrolyses in a fire"),
    entry(6, PIGMENT_BLACK, Form::Pure, "Finely ground carbon black for dark concrete and roofs"),
    block_entry(6, OIL_SAND, Form::Ore, "Bitumen-bearing sand pumped for crude hydrocarbons"),
    block_entry(6, GAS_SHALE, Form::Ore, "Deep organic shale tapped for natural gas"),
    entry(6, CRUDE_BARREL, Form::Liquid, "Unrefined mixture of liquid hydrocarbons"),
    entry(6, RAW_GAS_BARREL, Form::Gas, "Sour natural gas before H₂S sweetening"),
    entry(6, NAPHTHA_BARREL, Form::Liquid, "Light hydrocarbon cut refined into plastics"),
    entry(6, DIESEL_BARREL, Form::Liquid, "Middle distillate fuel for diesel generators"),
    entry(6, HEAVY_OIL_BARREL, Form::Liquid, "Long-chain cut cracked to naphtha or made into lubricant"),
    entry(6, LUBRICANT_BARREL, Form::Liquid, "Heavy hydrocarbon oil for rotating machinery"),
    entry(6, PLASTIC, Form::Compound, "Polyethylene / polymer chains from naphtha or ethylene"),
    entry(6, STEEL_INGOT, Form::Alloy, "Iron alloyed with ~0.5–1.5% carbon for high strength"),
    // 8 — Oxygen
    entry(8, OXYGEN_BARREL, Form::Gas, "O₂ gas split from water by electrolysis"),
    entry(8, PURE_WATER_BARREL, Form::Liquid, "H₂O purified for chip fabrication"),
    entry(8, QUICKLIME, Form::Oxide, "CaO — calcined limestone flux and cement binder"),
    block_entry(8, GLASS, Form::Oxide, "Amorphous SiO₂ melted from quartz"),
    entry(8, ROCKET_FUEL_BARREL, Form::Liquid, "Oxygen-rich propellant for orbital rockets"),
    // 13 — Aluminium
    block_entry(13, BAUXITE_ORE, Form::Ore, "Hydrated aluminium oxide ore found in arid provinces"),
    entry(13, CRUSHED_BAUXITE, Form::Ore, "Crushed ore ready for washing or electrolysis"),
    entry(13, WASHED_BAUXITE, Form::Oxide, "Cleaned alumina-rich concentrate"),
    entry(13, ALUMINIUM_INGOT, Form::Pure, "Smelted via Hall–Héroult electrolysis"),
    entry(13, ALUMINIUM_PLATE, Form::Pure, "Rolled lightweight sheet metal"),
    entry(13, LIGHT_FRAME, Form::Alloy, "Aerospace structural frame for rockets and satellites"),
    // 14 — Silicon
    block_entry(14, QUARTZ_ORE, Form::Ore, "Crystalline SiO₂ — feedstock for glass and silicon"),
    block_entry(14, GLASS, Form::Oxide, "Transparent amorphous silica"),
    entry(14, SILICON, Form::Pure, "Reduced from quartz with carbon in an electric arc furnace"),
    entry(14, WAFER, Form::Pure, "Ultra-pure monocrystalline silicon slice"),
    entry(14, CIRCUIT, Form::Compound, "Integrated electronic board built on silicon"),
    entry(14, PROCESSOR, Form::Compound, "High-density logic chip for drones and automation"),
    // 16 — Sulfur
    entry(16, SULFUR, Form::Pure, "Elemental sulfur recovered from oil refining and gas sweetening"),
    entry(16, ACID_BARREL, Form::Compound, "H₂SO₄ — sulfuric acid, the workhorse of chemical industry"),
    entry(16, PIGMENT_YELLOW, Form::Compound, "Bright yellow sulfur-based mineral pigment"),
    // 20 — Calcium
    block_entry(20, LIMESTONE, Form::Carbonate, "CaCO₃ sedimentary rock — source of lime"),
    entry(20, QUICKLIME, Form::Oxide, "CaO — produced by heating limestone to drive off CO₂"),
    entry(20, PIGMENT_WHITE, Form::Oxide, "Ground lime white for bright architectural concrete"),
    block_entry(20, CONCRETE, Form::Compound, "Calcium silicate binder cast with sand and stone"),
    // 26 — Iron
    block_entry(26, IRON_ORE, Form::Ore, "Iron oxide vein — foundation of early industry"),
    entry(26, CRUSHED_IRON, Form::Ore, "Mechanically crushed ore for higher smelter yield"),
    entry(26, WASHED_IRON, Form::Ore, "Water-washed concentrate for maximum metal recovery"),
    entry(26, SCRAP_METAL, Form::Alloy, "Reclaimed iron salvage from ancient ruins"),
    entry(26, IRON_INGOT, Form::Pure, "Smelted metallic iron"),
    entry(26, IRON_PLATE, Form::Pure, "Pressed sheet iron for belts and machines"),
    entry(26, STEEL_INGOT, Form::Alloy, "Blast-furnace iron-carbon alloy"),
    entry(26, STEEL_PLATE, Form::Alloy, "Heavy structural steel plate"),
    entry(26, PIGMENT_RED, Form::Oxide, "Fe₂O₃ — iron(III) oxide (rust red) pigment"),
    // 29 — Copper
    block_entry(29, COPPER_ORE, Form::Ore, "Copper-bearing vein prized for electrical conductivity"),
    entry(29, CRUSHED_COPPER, Form::Ore, "Crushed ore for boosted smelting output"),
    entry(29, WASHED_COPPER, Form::Ore, "Cleaned copper concentrate"),
    entry(29, COPPER_INGOT, Form::Pure, "Ductile red metal"),
    entry(29, COPPER_WIRE, Form::Pure, "Drawn conductor for power grids, motors and circuits"),
    entry(29, PIGMENT_GREEN, Form::Carbonate, "Copper carbonate (malachite / verdigris) green pigment"),
    entry(29, PIGMENT_BLUE, Form::Compound, "CuSO₄ — copper(II) sulfate blue pigment"),
    // 92 — Uranium
    block_entry(92, URANIUM_ORE, Form::Ore, "Deep radioactive ore found in distant lodes"),
    entry(92, FUEL_CELL, Form::Pure, "Enriched uranium fuel rod powering nuclear reactors"),
];

/// Facts and in-game roles for every element reachable in this version of OpenCraft.
pub const ELEMENTS: &[ElementInfo] = &[
    ElementInfo {
        z: 1,
        fact: "Lightest and most abundant element in the universe; makes up ~75% of all normal matter.",
        role: "Split from water or reformed from methane to fuel rockets, crackers and clean chemistry.",
    },
    ElementInfo {
        z: 6,
        fact: "Forms four covalent bonds, creating the backbone of fuels, polymers, steel and all known life.",
        role: "Powers early furnaces as coal or charcoal, strengthens steel, and drives the oil and gas chains.",
    },
    ElementInfo {
        z: 8,
        fact: "Makes up 21% of Earth's atmosphere and nearly half of the planet's crust by mass in oxide rocks.",
        role: "Bound in silica and lime, and split from water as pure O₂ to oxidise orbital rocket fuel.",
    },
    ElementInfo {
        z: 13,
        fact: "Most abundant metal in Earth's crust, yet locked so tightly to oxygen that electrolysis is needed to free it.",
        role: "Refined from bauxite in electrolytic cells for batteries, drones, light frames and satellites.",
    },
    ElementInfo {
        z: 14,
        fact: "A metalloid semiconductor; ultra-pure silicon crystals switch billions of transistors per second.",
        role: "Reduced from quartz in arc furnaces, sliced into wafers and etched into circuits and AI chips.",
    },
    ElementInfo {
        z: 16,
        fact: "Yellow nonmetal recovered from crude oil and sour gas; most of the world's supply becomes sulfuric acid.",
        role: "Captured during refining and gas sweetening to make acid for wafers, pigments and chemical plants.",
    },
    ElementInfo {
        z: 20,
        fact: "Reactive alkaline-earth metal locked in limestone (CaCO₃); heating drives off CO₂ to leave quicklime (CaO).",
        role: "Fluxes blast furnaces for steel, binds concrete, and grinds into bright white pigment.",
    },
    ElementInfo {
        z: 26,
        fact: "Earth's core is mostly iron; alloying it with a trace of carbon turns soft metal into structural steel.",
        role: "Backbone of belts, machines, rails and steel structures from the first smelter to the launch pad.",
    },
    ElementInfo {
        z: 29,
        fact: "Second only to silver in electrical conductivity and resists corrosion with a green carbonate patina.",
        role: "Drawn into wires, motors and circuits, and reacted with acid to synthesise vivid blue pigment.",
    },
    ElementInfo {
        z: 92,
        fact: "Heaviest naturally occurring primordial element; splitting one gram of U-235 rivals tonnes of coal.",
        role: "Mined from deep lodes and enriched in centrifuges into fuel cells for high-output nuclear reactors.",
    },
];

/// All 16 pigment items (hand-ground + chemical mixes), used by the pigment achievement.
pub const ALL_PIGMENTS: &[ItemId] = &[
    PIGMENT_RED,
    PIGMENT_GREEN,
    PIGMENT_BLACK,
    PIGMENT_WHITE,
    PIGMENT_YELLOW,
    PIGMENT_BLUE,
    item::PIGMENT_ORANGE,
    item::PIGMENT_PURPLE,
    item::PIGMENT_CYAN,
    item::PIGMENT_LIME,
    item::PIGMENT_PINK,
    item::PIGMENT_SKY,
    item::PIGMENT_GREY,
    item::PIGMENT_BROWN,
    item::PIGMENT_MAROON,
    item::PIGMENT_NAVY,
];

/// Every raw ore block as an `ItemId`.
pub const ALL_ORES: &[ItemId] = &[
    ItemId::block(COAL_ORE),
    ItemId::block(IRON_ORE),
    ItemId::block(COPPER_ORE),
    ItemId::block(LIMESTONE),
    ItemId::block(QUARTZ_ORE),
    ItemId::block(BAUXITE_ORE),
    ItemId::block(OIL_SAND),
    ItemId::block(URANIUM_ORE),
    ItemId::block(GAS_SHALE),
];

/// Every satellite item that can be launched into orbit.
pub const ALL_SATELLITES: &[ItemId] =
    &[item::SURVEY_SATELLITE, item::COMMS_SATELLITE, item::POWER_SATELLITE, item::SCIENCE_SATELLITE];

pub fn element_info(z: u8) -> Option<&'static ElementInfo> {
    ELEMENTS.iter().find(|e| e.z == z)
}
