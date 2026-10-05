//! The techs of Milestone 10 (fluids and chemistry), as data (appended after `distance.rs`'s in `techs::TECHS`, so the
//! order here is the saved index order: add rows at the bottom, never reorder). Needs name earlier indices (Fluid
//! Handling 6, Electronics 17); this table's own start at 42 (Oil Processing 42, Refining 43, Plastics 44, Acids and Lubricants 45, Diesel Power 46, Electrolysis 47, Ore Washing 48, Research Center 49, Hydropower 50, Hoists 51, Nuclear Power 52, Gold Science 53, Mk5 Machines 54; Steam Power 14).

use crate::block::{
    ASSEMBLER, BLAST_FURNACE, CENTRIFUGE, CHEMICAL_PLANT, CONSTRUCTOR, CRACKER, DIESEL_GENERATOR, DRONE_PORT,
    ELECTROLYSER, HOIST, MINER, PUMPJACK, REACTOR, REFINERY, RESEARCH_CENTER, SMELTER, WASHER, WATER_WHEEL, WINCH,
};
use crate::item::{BLUE_PACK, EMPTY_CANISTER, GOLD_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK};
use crate::recipes::{
    ACID_RECIPE, CANISTER_MACHINE_RECIPE, CRACK_RECIPE, DISTIL_RECIPE, ELECTROLYSE_RECIPE, ENRICH_RECIPE, GOLD_RECIPES,
    LUBRICANT_RECIPE, PLASTIC_RECIPE, WASH_RECIPES,
};

use super::{r, Tech, Unlock};

pub const CHEMISTRY: [Tech; 13] = [
    Tech {
        name: "Oil Processing",
        blurb: "Pumpjacks (90 kW) drill down to an oil reservoir and fill canisters with crude oil, a steel drum each \
                that belts carry like any item. Press empty canisters from steel plates, by hand or in a constructor. \
                Oil sand burns as a weak fuel; refining comes next.",
        needs: &[6, 17],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 40.0,
        unlocks: &[r(PUMPJACK), Unlock::Recipe(EMPTY_CANISTER), Unlock::MachineRecipe(CANISTER_MACHINE_RECIPE)],
    },
    Tech {
        name: "Refining",
        blurb:
            "The refinery (150 kW, a water pipe) distils 3 crude oil canisters into a naphtha, a diesel and a heavy \
                oil canister and a sulfur; the cracker (90 kW, a water pipe) splits 2 heavy oil canisters into a \
                naphtha and a diesel. Every stream must be used or stored, or the plant stops.",
        needs: &[42],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 160,
        seconds: 40.0,
        unlocks: &[r(REFINERY), r(CRACKER), Unlock::MachineRecipe(DISTIL_RECIPE), Unlock::MachineRecipe(CRACK_RECIPE)],
    },
    Tech {
        name: "Plastics",
        blurb:
            "The chemical plant (120 kW) turns 2 naphtha canisters and a coal into 4 plastic in 4 seconds and gives \
                the two empty canisters back. Choose what it makes in its panel.",
        needs: &[43],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 40.0,
        unlocks: &[r(CHEMICAL_PLANT), Unlock::MachineRecipe(PLASTIC_RECIPE)],
    },
    Tech {
        name: "Acids and Lubricants",
        blurb:
            "The chemical plant fills an empty canister with acid from a sulfur and a unit of water (3 seconds), or \
                turns a heavy oil canister and an empty one into 2 lubricant canisters.",
        needs: &[44],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 40.0,
        unlocks: &[Unlock::MachineRecipe(ACID_RECIPE), Unlock::MachineRecipe(LUBRICANT_RECIPE)],
    },
    Tech {
        name: "Diesel Power",
        blurb: "The diesel generator gives 400 kW from diesel canisters, 100 seconds each at full load, and hands the \
                empty canisters back. It burns only what the grid asks for, after solar, coal and steam.",
        needs: &[43],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 40.0,
        unlocks: &[r(DIESEL_GENERATOR)],
    },
    Tech {
        name: "Electrolysis",
        blurb:
            "The electrolyser (500 kW, a water pipe) splits water: 3 empty canisters become 2 hydrogen and an oxygen \
                canister in 6 seconds. Both gases must be used or stored.",
        needs: &[45],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 160,
        seconds: 40.0,
        unlocks: &[r(ELECTROLYSER), Unlock::MachineRecipe(ELECTROLYSE_RECIPE)],
    },
    Tech {
        name: "Ore Washing",
        blurb:
            "The washer (60 kW, a water pipe) cleans crushed iron, copper or bauxite: 3 crushed ore and a unit of \
                water make 4 washed ore and a tailings block, which smelt one for one. Raw ore does not fit, so the \
                crusher stays first: 2.0 ingots an ore washed against 1.5 crushed. The crusher grinds tailings to sand.",
        needs: &[6, 15, 36],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 160,
        seconds: 40.0,
        unlocks: &[
            r(WASHER),
            Unlock::MachineRecipe(WASH_RECIPES[0]),
            Unlock::MachineRecipe(WASH_RECIPES[1]),
            Unlock::MachineRecipe(WASH_RECIPES[2]),
            Unlock::MachineRecipe(WASH_RECIPES[3]),
            Unlock::MachineRecipe(WASH_RECIPES[4]),
            Unlock::MachineRecipe(WASH_RECIPES[5]),
            Unlock::MachineRecipe(WASH_RECIPES[6]),
        ],
    },
    Tech {
        name: "Research Center",
        blurb: "A 2×2×2 research building with eight pack slots, twice a lab's speed and belts on every side. A small \
                lab holds only the four older packs: techs that use a newer pack are researched in a center.",
        needs: &[17],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 40.0,
        unlocks: &[r(RESEARCH_CENTER)],
    },
    Tech {
        name: "Hydropower",
        blurb: "The water wheel (3×3×1) stands beside or in water and turns it into power: 2 kW for every water block \
                that touches it (4 for flowing water), up to 48 kW, with no fuel and day and night. A river with a \
                weir, or a pool piped from a pump, drives it. Hang it on a power pole.",
        needs: &[14],
        packs: &[RED_PACK, GREEN_PACK],
        units: 60,
        seconds: 30.0,
        unlocks: &[r(WATER_WHEEL)],
    },
    Tech {
        name: "Hoists",
        blurb: "Hoist shafts are steel climbing frames; a winch (20 kW) beside the top block makes the shaft carry you \
                at up to 9 blocks a second, three times a ladder, for the deep lodes. Belt lifts still carry the ore up.",
        needs: &[17],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 100,
        seconds: 30.0,
        unlocks: &[r(HOIST), r(WINCH)],
    },
    Tech {
        name: "Nuclear Power",
        blurb: "The centrifuge (200 kW) turns 4 uranium ore and a steel plate into a fuel cell; the reactor gives up to \
                2 MW, a cell lasting 150 seconds at full load, and burns only what the grid asks for. It needs \
                coolant: pipe water to its blue inlet, or it overheats and shuts down until it has cooled.",
        needs: &[14, 43],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 200,
        seconds: 45.0,
        unlocks: &[r(CENTRIFUGE), r(REACTOR), Unlock::MachineRecipe(ENRICH_RECIPE)],
    },
    Tech {
        name: "Gold Science",
        blurb: "Gold science packs, assembled from a plastic, a battery and a processor (20 seconds, two packs): the \
                packs of the chemical era. A small lab has no slot for them: techs that use them are researched in a \
                research center.",
        needs: &[24, 36, 44, 49],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 240,
        seconds: 40.0,
        unlocks: &[Unlock::MachineRecipe(GOLD_RECIPES[0])],
    },
    Tech {
        name: "Mk5 Machines",
        blurb: "Gold kits (a processor, 2 aluminium plates and a plastic make 4, in an assembler) raise miners, \
                smelters, constructors, assemblers, blast furnaces, drone ports and research centers to Mk5: eight \
                times as fast (a center ten), miners recover 97%, ports keep 24 drones 128 blocks out.",
        needs: &[20, 29, 53],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 200,
        seconds: 45.0,
        unlocks: &[
            Unlock::MachineRecipe(GOLD_RECIPES[1]),
            Unlock::Upgrade(MINER, 4),
            Unlock::Upgrade(SMELTER, 4),
            Unlock::Upgrade(CONSTRUCTOR, 4),
            Unlock::Upgrade(ASSEMBLER, 4),
            Unlock::Upgrade(BLAST_FURNACE, 4),
            Unlock::Upgrade(DRONE_PORT, 4),
            Unlock::Upgrade(RESEARCH_CENTER, 4),
        ],
    },
];
