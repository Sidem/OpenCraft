//! Violet ring (33..60) of [`super::TECHS`].

use crate::block::{
    ASSEMBLER, BELT, BLAST_FURNACE, CENTRIFUGE, CHEMICAL_PLANT, CONSTRUCTOR, CRACKER, DIESEL_GENERATOR, DRONE_PORT,
    ELECTROLYSER, ELECTROLYTIC_CELL, GAS_PLANT, GAS_TURBINE, GAS_WELL, LAB, LOADING_DOCK, LONG_POLE, MINER, POLE,
    PUMPJACK, RAIL, REACTOR, REFINERY, RESEARCH_CENTER, SMELTER, STEAM_REFORMER, STORAGE, THUMPER, UNDERPASS_IN,
    UNLOADING_DOCK, WELLHEAD_SEPARATOR,
};
use crate::item::{
    BLUE_PACK, EMPTY_BARREL, EXO_FRAME, FLAMETHROWER, GREEN_PACK, HAULER_PACK_MK4, HOVER_PACK, JETPACK, LOCOMOTIVE,
    PERSONAL_DRONE, PLANNER, RAIL_SIGNAL, RED_PACK, SERVO_BOOTS, TOOL_BELT_MK4, VIOLET_PACK, WAGON, WORK_GLOVES_MK4,
};
use crate::recipes::{
    ACID_RECIPE, BARREL_MACHINE_RECIPE, BAUXITE_RECIPES, CARGO_RECIPE, CRACK_RECIPE, DISTIL_RECIPE, DRONE_RECIPES,
    DYE_FIRST, DYE_PLANT_ROWS, ELECTROLYSE_RECIPE, ENRICH_RECIPE, ETHANE_CRACK_RECIPE, GAS_PROCESS_RECIPE, GLASS_FIRST,
    GLASS_PRIMARY_COUNT, GLASS_RAW_FIRST, GLASS_ROWS, LONG_POLE_ROW, LPG_CRACK_RECIPE, LUBRICANT_RECIPE,
    PLASTIC_RECIPE, POLYETHYLENE_RECIPE, RAIL_ROW, REFORM_RECIPE, VIOLET_RECIPES,
};

use super::ids::*;
use super::rings::{Ring, Wedge};
use super::{r, Feature, Tech, Unlock};

/// Pigments unlocks: the chemical plant's 11 mixed pigments plus the 12 stained glass hues (Yellow + 11 mixed, ×3
/// silica batches = 36 rows) in the Glass Kiln.
const DYE_UNLOCKS: [Unlock; DYE_PLANT_ROWS as usize + 3 * (GLASS_ROWS - GLASS_PRIMARY_COUNT) as usize] = {
    let mut out =
        [Unlock::MachineRecipe(DYE_FIRST); DYE_PLANT_ROWS as usize + 3 * (GLASS_ROWS - GLASS_PRIMARY_COUNT) as usize];
    let mut k = 0;
    let mut i = 0;
    while i < DYE_PLANT_ROWS as usize {
        out[k] = Unlock::MachineRecipe(DYE_FIRST + i as u16);
        k += 1;
        i += 1;
    }
    let mut j = GLASS_PRIMARY_COUNT as usize;
    while j < GLASS_ROWS as usize {
        out[k] = Unlock::MachineRecipe(GLASS_FIRST + j as u16);
        out[k + 1] = Unlock::MachineRecipe(GLASS_RAW_FIRST + j as u16);
        out[k + 2] = Unlock::MachineRecipe(GLASS_RAW_FIRST + GLASS_ROWS + j as u16);
        k += 3;
        j += 1;
    }
    out
};

pub(super) const VIOLET: [Tech; 27] = [
    Tech {
        name: "Mk4 Kits",
        blurb: "Violet kits, made in an assembler, raise belts, underpasses, substations, boxes, miners, smelters, \
                constructors, assemblers, blast furnaces, labs, seismic thumpers, hauler packs, work gloves and \
                the Tool Belt to Mk4: five times as fast, 81 pack slots, +100% hand crafting and a rock drill.",
        ring: Ring::Violet,
        wedge: Wedge::Workshop,
        deed: None,
        needs: &[ELECTRONICS, MK3_KITS, STEEL_TOOLS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 30.0,
        unlocks: &[
            Unlock::MachineRecipe(VIOLET_RECIPES[1]),
            Unlock::Recipe(HAULER_PACK_MK4),
            Unlock::Recipe(WORK_GLOVES_MK4),
            Unlock::Recipe(TOOL_BELT_MK4),
            Unlock::Upgrade(BELT, 3),
            Unlock::Upgrade(UNDERPASS_IN, 3),
            Unlock::Upgrade(POLE, 3),
            Unlock::Upgrade(STORAGE, 3),
            Unlock::Upgrade(MINER, 3),
            Unlock::Upgrade(SMELTER, 3),
            Unlock::Upgrade(CONSTRUCTOR, 3),
            Unlock::Upgrade(ASSEMBLER, 3),
            Unlock::Upgrade(BLAST_FURNACE, 3),
            Unlock::Upgrade(LAB, 3),
            Unlock::Upgrade(THUMPER, 1),
        ],
    },
    Tech {
        name: "Processors",
        blurb: "Assemblers solder processors from circuits, silicon and steel: the brains of everything that thinks \
                for itself.",
        ring: Ring::Violet,
        wedge: Wedge::Computing,
        deed: None,
        needs: &[ELECTRONICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 100,
        seconds: 30.0,
        unlocks: &[Unlock::MachineRecipe(DRONE_RECIPES[0])],
    },
    Tech {
        name: "Robotics",
        blurb: "Servos (a motor, circuits and gears) and the actuators built from them: the muscles of a machine \
                that moves. Assemblers only.",
        ring: Ring::Violet,
        wedge: Wedge::RobotsDrones,
        deed: None,
        needs: &[ELECTRONICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 30.0,
        unlocks: &[Unlock::MachineRecipe(DRONE_RECIPES[1]), Unlock::MachineRecipe(DRONE_RECIPES[2])],
    },
    Tech {
        name: "Construction Drones",
        blurb: "Drone cells, guidance modules, drones and the drone port (3×3 pad): drones automatically build your \
                ghosts and tear down what you mark from storage boxes beside the pad.",
        ring: Ring::Violet,
        wedge: Wedge::RobotsDrones,
        deed: None,
        needs: &[PROCESSORS, ROBOTICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 240,
        seconds: 40.0,
        unlocks: &[
            Unlock::MachineRecipe(DRONE_RECIPES[3]),
            Unlock::MachineRecipe(DRONE_RECIPES[4]),
            Unlock::MachineRecipe(DRONE_RECIPES[5]),
            r(DRONE_PORT),
        ],
    },
    Tech {
        name: "Swarm Logistics",
        blurb: "Kits upgrade drone ports in place: Mk2 keeps 8 drones and reaches 48 blocks, Mk3 12 and 64, Mk4 16 \
                and 96.",
        ring: Ring::Violet,
        wedge: Wedge::RobotsDrones,
        deed: None,
        needs: &[CONSTRUCTION_DRONES, MK4_KITS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 200,
        seconds: 40.0,
        unlocks: &[Unlock::Upgrade(DRONE_PORT, 1), Unlock::Upgrade(DRONE_PORT, 2), Unlock::Upgrade(DRONE_PORT, 3)],
    },
    Tech {
        name: "Jetpack",
        blurb: "Four steel plates, two motors and two circuits: a coal-burning pack, worn in its own slot, that \
                lifts you while you hold jump. Ten seconds of thrust a coal.",
        ring: Ring::Violet,
        wedge: Wedge::YourGear,
        deed: None,
        needs: &[ELECTRONICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 30.0,
        unlocks: &[Unlock::Recipe(JETPACK)],
    },
    Tech {
        name: "Personal Drone",
        blurb: "A drone and two circuits make a companion that fetches what you ask for from the storage boxes \
                near you.",
        ring: Ring::Violet,
        wedge: Wedge::YourGear,
        deed: None,
        needs: &[CONSTRUCTION_DRONES],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 160,
        seconds: 35.0,
        unlocks: &[Unlock::Recipe(PERSONAL_DRONE)],
    },
    Tech {
        name: "Earthworks",
        blurb: "The planner marks an area to dig, fill or flatten and shows what that would move. Drone ports do the \
                work: they dig into the storage boxes beside the pad and fill from them.",
        ring: Ring::Violet,
        wedge: Wedge::RobotsDrones,
        deed: None,
        needs: &[CONSTRUCTION_DRONES],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 35.0,
        unlocks: &[Unlock::Recipe(PLANNER)],
    },
    Tech {
        name: "Exosuit",
        blurb: "Powered gear built from servos and actuators: servo boots (step up a whole block) and an exo \
                frame (reach 8 blocks).",
        ring: Ring::Violet,
        wedge: Wedge::YourGear,
        deed: None,
        needs: &[ROBOTICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 30.0,
        unlocks: &[Unlock::Recipe(SERVO_BOOTS), Unlock::Recipe(EXO_FRAME)],
    },
    Tech {
        name: "Bauxite Processing",
        blurb: "Crushers grind bauxite ore, and electrolytic cells (300 kW) turn 2 crushed bauxite and a quicklime \
                into an aluminium ingot and a slag. Constructors press plates; assemblers build batteries.",
        ring: Ring::Violet,
        wedge: Wedge::OresMetals,
        deed: None,
        needs: &[ORE_CRUSHING, ELECTRONICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 30.0,
        unlocks: &[
            r(ELECTROLYTIC_CELL),
            Unlock::MachineRecipe(BAUXITE_RECIPES[0]),
            Unlock::MachineRecipe(BAUXITE_RECIPES[1]),
            Unlock::MachineRecipe(BAUXITE_RECIPES[2]),
            Unlock::MachineRecipe(BAUXITE_RECIPES[3]),
        ],
    },
    Tech {
        name: "Railways",
        blurb: "Rails, locomotives, cargo wagons, rail signals and loading/unloading docks: smooth track between rail \
                nodes up to 32 blocks apart, hauling bulk freight across the map.",
        ring: Ring::Violet,
        wedge: Wedge::Logistics,
        deed: None,
        needs: &[ELECTRONICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 180,
        seconds: 35.0,
        unlocks: &[
            r(RAIL),
            Unlock::MachineRecipe(RAIL_ROW),
            Unlock::Recipe(LOCOMOTIVE),
            Unlock::Recipe(WAGON),
            Unlock::Recipe(RAIL_SIGNAL),
            r(LOADING_DOCK),
            r(UNLOADING_DOCK),
        ],
    },
    Tech {
        name: "Hover Pack",
        blurb: "Six aluminium plates, two motors, four batteries and two processors make a pack that hovers: hold \
                jump in the air to stay up, and glide at 8 blocks a second. Charges near a power pole.",
        ring: Ring::Violet,
        wedge: Wedge::YourGear,
        deed: None,
        needs: &[BAUXITE_PROCESSING, PROCESSORS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 160,
        seconds: 40.0,
        unlocks: &[Unlock::Recipe(HOVER_PACK)],
    },
    Tech {
        name: "Cargo Drones",
        blurb: "Assemblers build cargo drones from a drone, 4 aluminium plates and 2 batteries. Keep them in a drone \
                port and link two ports: they haul stacks by air, burning batteries for distance.",
        ring: Ring::Violet,
        wedge: Wedge::RobotsDrones,
        deed: None,
        needs: &[CONSTRUCTION_DRONES, BAUXITE_PROCESSING],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 200,
        seconds: 40.0,
        unlocks: &[Unlock::MachineRecipe(CARGO_RECIPE)],
    },
    Tech {
        name: "Oil Refining",
        blurb: "Pumpjacks (90 kW) fill steel barrels with crude oil; refineries (150 kW, water pipe) distil 3 crude \
                barrels into naphtha, diesel, heavy oil and sulfur; crackers split heavy oil into naphtha and diesel.",
        ring: Ring::Violet,
        wedge: Wedge::Chemistry,
        deed: None,
        needs: &[FLUID_HANDLING, ELECTRONICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 160,
        seconds: 40.0,
        unlocks: &[
            r(PUMPJACK),
            Unlock::Recipe(EMPTY_BARREL),
            Unlock::MachineRecipe(BARREL_MACHINE_RECIPE),
            r(REFINERY),
            r(CRACKER),
            Unlock::MachineRecipe(DISTIL_RECIPE),
            Unlock::MachineRecipe(CRACK_RECIPE),
        ],
    },
    Tech {
        name: "Plastics",
        blurb: "The chemical plant (120 kW) turns 2 naphtha barrels and a coal into 4 plastic in 4 seconds and gives \
                the two empty barrels back.",
        ring: Ring::Violet,
        wedge: Wedge::Chemistry,
        deed: None,
        needs: &[OIL_REFINING],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 40.0,
        unlocks: &[r(CHEMICAL_PLANT), Unlock::MachineRecipe(PLASTIC_RECIPE)],
    },
    Tech {
        name: "Acids and Lubricants",
        blurb: "The chemical plant fills an empty barrel with acid from a sulfur and a unit of water (3 seconds), or \
                turns a heavy oil barrel and an empty one into 2 lubricant barrels.",
        ring: Ring::Violet,
        wedge: Wedge::Chemistry,
        deed: None,
        needs: &[PLASTICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 40.0,
        unlocks: &[Unlock::MachineRecipe(ACID_RECIPE), Unlock::MachineRecipe(LUBRICANT_RECIPE)],
    },
    Tech {
        name: "Diesel Power",
        blurb: "The diesel generator gives 400 kW from diesel barrels, 50 seconds each at full load, and hands the \
                empty barrels back. Burns only what the grid asks for, after solar, coal and steam.",
        ring: Ring::Violet,
        wedge: Wedge::Power,
        deed: None,
        needs: &[OIL_REFINING],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 40.0,
        unlocks: &[r(DIESEL_GENERATOR)],
    },
    Tech {
        name: "Electrolysis",
        blurb: "The electrolyser (500 kW, a water pipe) splits water: 3 empty barrels become 2 hydrogen and an oxygen \
                barrel in 6 seconds. Both gases must be used or stored.",
        ring: Ring::Violet,
        wedge: Wedge::Chemistry,
        deed: None,
        needs: &[OIL_REFINING, FLUID_HANDLING],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 160,
        seconds: 40.0,
        unlocks: &[r(ELECTROLYSER), Unlock::MachineRecipe(ELECTROLYSE_RECIPE)],
    },
    Tech {
        name: "Research Center",
        blurb: "A 2×2×2 research building with eight pack slots, twice a lab's speed and belts on every side. Techs \
                that use gold packs are researched in a center.",
        ring: Ring::Violet,
        wedge: Wedge::Workshop,
        deed: None,
        needs: &[ELECTRONICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 40.0,
        unlocks: &[r(RESEARCH_CENTER)],
    },
    Tech {
        name: "Nuclear Power",
        blurb:
            "The centrifuge (200 kW) turns 4 uranium ore and a steel plate into a fuel cell; the reactor gives up to \
                2 MW, a cell lasting 150 seconds at full load. Needs piped cooling water.",
        ring: Ring::Violet,
        wedge: Wedge::Power,
        deed: None,
        needs: &[FLUID_HANDLING, ELECTRONICS, PLASTICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 200,
        seconds: 45.0,
        unlocks: &[r(CENTRIFUGE), r(REACTOR), Unlock::MachineRecipe(ENRICH_RECIPE)],
    },
    Tech {
        name: "Auto-Routing",
        blurb: "Drag a belt line onto a machine and it finds its own way there: round walls, up and down steps. In \
                ghost mode (B) a dragged line is planted as ghosts for the drones to build.",
        ring: Ring::Violet,
        wedge: Wedge::Logistics,
        deed: None,
        needs: &[CONSTRUCTION_DRONES],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 150,
        seconds: 40.0,
        unlocks: &[Unlock::Feature(Feature::AutoRoute)],
    },
    Tech {
        name: "AI Survey",
        blurb: "The maps mark where ore probably lies, guessed from the stained soil you have seen within 256 blocks: \
                faint rings in the ore's colour, bigger over more stain.",
        ring: Ring::Violet,
        wedge: Wedge::Survey,
        deed: None,
        needs: &[ADVANCED_SCANNING, PROCESSORS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 100,
        seconds: 40.0,
        unlocks: &[Unlock::Feature(Feature::AiSurvey)],
    },
    Tech {
        name: "Long-Distance Power",
        blurb: "Lattice towers, 2×2 and 3 high, that link to each other 160 blocks apart, to carry power across the \
                wilderness to far mines and oil wells.",
        ring: Ring::Violet,
        wedge: Wedge::Power,
        deed: None,
        needs: &[BAUXITE_PROCESSING],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 150,
        seconds: 30.0,
        unlocks: &[r(LONG_POLE), Unlock::MachineRecipe(LONG_POLE_ROW)],
    },
    Tech {
        name: "Pigments",
        blurb: "The chemical plant makes blue pigment and mixes two pigments into ten more hues for the Paint Sprayer \
                and the Glass Kiln (including yellow stained glass from sulfur).",
        ring: Ring::Violet,
        wedge: Wedge::Chemistry,
        deed: None,
        needs: &[ACIDS_AND_LUBRICANTS, GLASS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 40.0,
        unlocks: &DYE_UNLOCKS,
    },
    Tech {
        name: "Natural Gas",
        blurb: "Tap deep natural gas shale fields and oil-field associated gas: gas wells, wellhead separators, gas \
                plants, 600 kW gas turbines, steam reformers for hydrogen and ethane/LPG cracking to polyethylene.",
        ring: Ring::Violet,
        wedge: Wedge::Chemistry,
        deed: None,
        needs: &[ACIDS_AND_LUBRICANTS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 180,
        seconds: 40.0,
        unlocks: &[
            r(GAS_WELL),
            r(GAS_PLANT),
            Unlock::MachineRecipe(GAS_PROCESS_RECIPE),
            r(WELLHEAD_SEPARATOR),
            r(GAS_TURBINE),
            r(STEAM_REFORMER),
            Unlock::MachineRecipe(REFORM_RECIPE),
            Unlock::MachineRecipe(ETHANE_CRACK_RECIPE),
            Unlock::MachineRecipe(LPG_CRACK_RECIPE),
            Unlock::MachineRecipe(POLYETHYLENE_RECIPE),
        ],
    },
    Tech {
        name: "Flamethrower",
        blurb: "A hand liquid-fuel torch for clearing weeds and burning grown trees down to charcoal. Runs on naphtha \
                or diesel barrels; never harms placed blocks, machines or houses.",
        ring: Ring::Violet,
        wedge: Wedge::YourGear,
        deed: None,
        needs: &[OIL_REFINING],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 30.0,
        unlocks: &[Unlock::Recipe(FLAMETHROWER)],
    },
    Tech {
        name: "Seismic Prospecting",
        blurb: "The Seismic Thumper (2×2×2, 40 kW): scan scrubbable 64-block vertical cross-sections and a 3D \
                subsurface cutaway down to bedrock. Upgrade to Mk2 (violet kits) and Mk3 (gold kits).",
        ring: Ring::Violet,
        wedge: Wedge::Survey,
        deed: None,
        needs: &[ELECTRONICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 30.0,
        unlocks: &[r(THUMPER)],
    },
];
