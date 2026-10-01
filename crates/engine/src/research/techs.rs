//! The tech tree as data ([`TECHS`]): every tech's packs, cost, prerequisites (by index) and unlocks.
//! Saves store progress by index, so append rows, never reorder (`research.rs` has the rules and the
//! progress state).

use crate::block::{
    ACCUMULATOR, ARC_FURNACE, ASSEMBLER, BELT, BLAST_FURNACE, BOILER, CONSTRUCTOR, CRUSHER, FILTER, GENERATOR, LAB,
    LIFT, MINER, OUTLET, PIPE, POLE, PUMP, QUARRY, SILO, SMELTER, SOLAR_PANEL, SPLITTER, STORAGE, TURBINE,
    UNDERPASS_IN, UNDERPASS_OUT,
};
use crate::item::{BLUE_PACK, GREEN_KIT, GREEN_PACK, RED_PACK, STEEL_AXE, STEEL_PICKAXE, STEEL_SHOVEL, VIOLET_PACK};
use crate::recipes::{
    ASSEMBLY_RECIPES, BLUE_RECIPES, BRICK_RECIPE, CRUSH_RECIPES, ELECTRONICS_RECIPES, GEAR_RECIPE, QUICKLIME_RECIPE,
    STEEL_RECIPES, VIOLET_RECIPES,
};

use super::{r, Tech, Unlock};
pub const TECHS: &[Tech] = &[
    Tech {
        name: "Belt Routing",
        blurb: "Splitters share items between belts; filters sort them.",
        needs: &[],
        packs: &[RED_PACK],
        units: 10,
        seconds: 5.0,
        unlocks: &[r(SPLITTER), r(FILTER)],
    },
    Tech {
        name: "Belt Lifts",
        blurb: "Lifts carry items straight up cliffs. (Belts climb and drop one-block steps without research.)",
        needs: &[0],
        packs: &[RED_PACK],
        units: 20,
        seconds: 5.0,
        unlocks: &[r(LIFT)],
    },
    Tech {
        name: "Green Science",
        blurb: "Green science packs, made from belts and screws, for the next techs.",
        needs: &[0],
        packs: &[RED_PACK],
        units: 30,
        seconds: 5.0,
        unlocks: &[Unlock::Recipe(GREEN_PACK)],
    },
    Tech {
        name: "Underpasses",
        blurb: "Belts that dive under a crossing belt and come back up.",
        needs: &[2],
        packs: &[RED_PACK, GREEN_PACK],
        units: 15,
        seconds: 10.0,
        unlocks: &[r(UNDERPASS_IN), r(UNDERPASS_OUT)],
    },
    // Was "Miner Mk2" (saves keep progress by index): kits replaced the separate Mk2 recipes.
    Tech {
        name: "Mechanics",
        blurb: "Gears and green kits. Hold kits and right-click a machine to make it Mk2: miners draw twice as fast \
                and keep 75% instead of 60%, smelters, constructors and labs work twice as fast, boxes hold 36 stacks, \
                pumps and quarries double up, generators give 100 kW, poles reach further.",
        needs: &[2],
        packs: &[RED_PACK, GREEN_PACK],
        units: 30,
        seconds: 10.0,
        unlocks: &[
            Unlock::MachineRecipe(GEAR_RECIPE),
            Unlock::Recipe(GREEN_KIT),
            Unlock::Upgrade(MINER, 1),
            Unlock::Upgrade(SMELTER, 1),
            Unlock::Upgrade(CONSTRUCTOR, 1),
            Unlock::Upgrade(POLE, 1),
            Unlock::Upgrade(STORAGE, 1),
            Unlock::Upgrade(PUMP, 1),
            Unlock::Upgrade(QUARRY, 1),
            Unlock::Upgrade(LAB, 1),
            Unlock::Upgrade(GENERATOR, 1),
        ],
    },
    // Was "Fast Belts".
    Tech {
        name: "Belt Mk2",
        blurb: "Green kits upgrade belts to Mk2, twice as fast: hold kits and drag along a belt line.",
        needs: &[4],
        packs: &[RED_PACK, GREEN_PACK],
        units: 20,
        seconds: 10.0,
        unlocks: &[Unlock::Upgrade(BELT, 1)],
    },
    Tech {
        name: "Fluid Handling",
        blurb: "Pumps lift water out of ponds and pits; pipes carry it to outlets that pour it out elsewhere.",
        needs: &[0],
        packs: &[RED_PACK],
        units: 15,
        seconds: 5.0,
        unlocks: &[r(PUMP), r(PIPE), r(OUTLET)],
    },
    Tech {
        name: "Masonry",
        blurb: "Smelters fire stone into bricks and limestone into quicklime.",
        needs: &[],
        packs: &[RED_PACK],
        units: 15,
        seconds: 5.0,
        unlocks: &[Unlock::MachineRecipe(BRICK_RECIPE), Unlock::MachineRecipe(QUICKLIME_RECIPE)],
    },
    Tech {
        name: "Assembly",
        blurb: "The assembler, a 2×2×2 machine that puts parts together: motors, concrete, and packs and kits \
                by the beltful.",
        needs: &[4],
        packs: &[RED_PACK, GREEN_PACK],
        units: 40,
        seconds: 10.0,
        unlocks: &[
            r(ASSEMBLER),
            Unlock::MachineRecipe(ASSEMBLY_RECIPES[0]),
            Unlock::MachineRecipe(ASSEMBLY_RECIPES[1]),
            Unlock::MachineRecipe(ASSEMBLY_RECIPES[2]),
            Unlock::MachineRecipe(ASSEMBLY_RECIPES[3]),
            Unlock::MachineRecipe(ASSEMBLY_RECIPES[4]),
            Unlock::MachineRecipe(ASSEMBLY_RECIPES[5]),
            Unlock::Upgrade(ASSEMBLER, 1),
        ],
    },
    Tech {
        name: "Steelmaking",
        blurb: "The blast furnace, a 2×2×3 furnace: iron ore, coal and quicklime in, steel out the front and slag \
                out the side. Constructors press steel into plates and beams.",
        needs: &[7, 8],
        packs: &[RED_PACK, GREEN_PACK],
        units: 50,
        seconds: 15.0,
        unlocks: &[
            r(BLAST_FURNACE),
            Unlock::MachineRecipe(STEEL_RECIPES[0]),
            Unlock::MachineRecipe(STEEL_RECIPES[1]),
            Unlock::MachineRecipe(STEEL_RECIPES[2]),
            Unlock::Upgrade(BLAST_FURNACE, 1),
        ],
    },
    Tech {
        name: "Blue Science",
        blurb: "Blue science packs, assembled from a motor, a steel plate and concrete: the packs of the next era.",
        needs: &[9],
        packs: &[RED_PACK, GREEN_PACK],
        units: 50,
        seconds: 15.0,
        unlocks: &[Unlock::MachineRecipe(BLUE_RECIPES[0])],
    },
    Tech {
        name: "Mk3 Logistics",
        blurb: "Blue kits, made in an assembler, and Mk3 belts (four times a Mk1), pylons that link 32 blocks and \
                boxes of 48 stacks.",
        needs: &[10],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 60,
        seconds: 20.0,
        unlocks: &[
            Unlock::MachineRecipe(BLUE_RECIPES[1]),
            Unlock::Upgrade(BELT, 2),
            Unlock::Upgrade(POLE, 2),
            Unlock::Upgrade(STORAGE, 2),
        ],
    },
    Tech {
        name: "Mk3 Machines",
        blurb: "Blue kits make miners, smelters, constructors, assemblers, blast furnaces, labs, pumps and quarries \
                Mk3: three times as fast, miners recover 85%, labs skip the packs of every fifth unit, and the \
                smelter goes electric.",
        needs: &[11],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 80,
        seconds: 20.0,
        unlocks: &[
            Unlock::Upgrade(MINER, 2),
            Unlock::Upgrade(SMELTER, 2),
            Unlock::Upgrade(CONSTRUCTOR, 2),
            Unlock::Upgrade(ASSEMBLER, 2),
            Unlock::Upgrade(BLAST_FURNACE, 2),
            Unlock::Upgrade(LAB, 2),
            Unlock::Upgrade(PUMP, 2),
            Unlock::Upgrade(QUARRY, 2),
        ],
    },
    Tech {
        name: "Steel Tools",
        blurb: "Steel pickaxes, axes and shovels: they last 1,500 blocks, dig six times as fast as bare hands and \
                keep 5 ore a block.",
        needs: &[9],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 30,
        seconds: 15.0,
        unlocks: &[Unlock::Recipe(STEEL_PICKAXE), Unlock::Recipe(STEEL_AXE), Unlock::Recipe(STEEL_SHOVEL)],
    },
    Tech {
        name: "Steam Power",
        blurb: "Boilers turn water and coal into steam, twice the energy of a generator's fire; steam turbines \
                turn it into up to 240 kW each. Pipe water to a boiler, belt it fuel, and set turbines against it.",
        needs: &[7, 10],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 60,
        seconds: 20.0,
        unlocks: &[r(BOILER), r(TURBINE)],
    },
    Tech {
        name: "Ore Crushing",
        blurb: "Crushers turn 2 iron or copper ore into 3 crushed ore, which smelt one for one: an ore gives one \
                and a half ingots. They also grind slag to sand.",
        needs: &[10],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 60,
        seconds: 20.0,
        unlocks: &[
            r(CRUSHER),
            Unlock::MachineRecipe(CRUSH_RECIPES[0]),
            Unlock::MachineRecipe(CRUSH_RECIPES[1]),
            Unlock::MachineRecipe(CRUSH_RECIPES[2]),
            Unlock::MachineRecipe(CRUSH_RECIPES[3]),
            Unlock::MachineRecipe(CRUSH_RECIPES[4]),
        ],
    },
    Tech {
        name: "Bulk Storage",
        blurb: "Silos: 2×2×3 stores of 144 stacks that take belts on every side and give to belts leading away.",
        needs: &[10],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 40,
        seconds: 20.0,
        unlocks: &[r(SILO)],
    },
    Tech {
        name: "Electronics",
        blurb: "Arc furnaces turn a quartz ore and a coal into silicon (120 kW), and assemblers make circuits from \
                silicon, copper wire and an iron plate. Quartz lies deep: scan for pale soil.",
        needs: &[10],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 80,
        seconds: 20.0,
        unlocks: &[
            r(ARC_FURNACE),
            Unlock::MachineRecipe(ELECTRONICS_RECIPES[0]),
            Unlock::MachineRecipe(ELECTRONICS_RECIPES[1]),
        ],
    },
    Tech {
        name: "Violet Science",
        blurb: "Violet science packs, assembled from two circuits, a steel beam and a motor: the packs of the \
                electronic era. Labs get a fourth slot for them.",
        needs: &[17],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 100,
        seconds: 25.0,
        unlocks: &[Unlock::MachineRecipe(VIOLET_RECIPES[0])],
    },
    Tech {
        name: "Mk4 Logistics",
        blurb: "Violet kits, made in an assembler, and Mk4 belts (eight times a Mk1) and substations that link 32 \
                blocks and power machines within 16.",
        needs: &[18],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 100,
        seconds: 30.0,
        unlocks: &[Unlock::MachineRecipe(VIOLET_RECIPES[1]), Unlock::Upgrade(BELT, 3), Unlock::Upgrade(POLE, 3)],
    },
    Tech {
        name: "Mk4 Machines",
        blurb: "Violet kits make miners, smelters, constructors, assemblers, blast furnaces and labs Mk4: five \
                times as fast (a lab four), miners recover 92%, and labs skip the packs of every third unit.",
        needs: &[19],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 30.0,
        unlocks: &[
            Unlock::Upgrade(MINER, 3),
            Unlock::Upgrade(SMELTER, 3),
            Unlock::Upgrade(CONSTRUCTOR, 3),
            Unlock::Upgrade(ASSEMBLER, 3),
            Unlock::Upgrade(BLAST_FURNACE, 3),
            Unlock::Upgrade(LAB, 3),
        ],
    },
    Tech {
        name: "Solar Power",
        blurb: "Solar panels give up to 10 kW by day and nothing at night; accumulators store 10 MJ of spare sun and \
                give it back after dark, before any generator burns fuel. Silicon and steel make them.",
        needs: &[17],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 80,
        seconds: 20.0,
        unlocks: &[r(SOLAR_PANEL), r(ACCUMULATOR)],
    },
];
