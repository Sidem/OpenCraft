//! Gold (60..72, including the three endless bonus techs at 65..68) and Sky (72..76) rings of [`super::TECHS`].

use crate::block::{
    AI_LAB, ASSEMBLER, BLAST_FURNACE, CHIP_FAB as CHIP_FAB_BLOCK, CONSTRUCTOR, COOLING_TOWER, DATACENTER,
    DATA_RECEIVER, DRONE_PORT, FIBRE_NODE, GROUND_STATION, LANDING_PAD, LASER_EMITTER, LASER_MIRROR, LASER_RECEIVER,
    LAUNCH_SITE, MINER, OPTIMIZER, POD_LAUNCHER, PUMPJACK, RECTENNA, RESEARCH_CENTER, SMELTER, STORAGE, SWARM_HUB,
    THUMPER,
};
use crate::item::{
    BLUE_PACK, CRAFT_GLOVES, GOLD_PACK, GREEN_PACK, HAULER_PACK_MK5, RED_PACK, TOOL_BELT_MK5, VIOLET_PACK,
};
use crate::recipes::{
    ACCELERATOR_RECIPE, COMMS_SATELLITE_RECIPES, DROP_POD_RECIPE, GOLD_RECIPES, POWER_SATELLITE_RECIPES,
    PURE_WATER_RECIPE, ROCKET_FUEL_RECIPE, ROCKET_PART_RECIPES, ROCKET_RECIPE, SCIENCE_SATELLITE_RECIPES,
    SURVEY_SATELLITE_RECIPE, WAFER_RECIPE,
};

use super::bonus::BONUS;
use super::ids::*;
use super::rings::{Ring, Wedge};
use super::{r, Tech, Unlock};

pub(super) const GOLD_SKY: [Tech; 16] = [
    // --- Gold ring (60..72) ---
    Tech {
        name: "Mk5 Kits",
        blurb: "Gold kits raise miners, smelters, constructors, assemblers, blast furnaces, drone ports, research \
                centers, storage boxes, pumpjacks, thumpers, hauler packs, work gloves and the Tool \
                Belt to their top tier: eight times as fast, 99 pack slots, Fabricator Gloves (+125% hand crafting) \
                and a laser cutter.",
        ring: Ring::Gold,
        wedge: Wedge::Workshop,
        deed: None,
        needs: &[MK4_KITS, PROCESSORS, BAUXITE_PROCESSING, PLASTICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 200,
        seconds: 45.0,
        unlocks: &[
            Unlock::MachineRecipe(GOLD_RECIPES[1]),
            Unlock::Recipe(HAULER_PACK_MK5),
            Unlock::Recipe(CRAFT_GLOVES),
            Unlock::Recipe(TOOL_BELT_MK5),
            Unlock::Upgrade(MINER, 4),
            Unlock::Upgrade(SMELTER, 4),
            Unlock::Upgrade(CONSTRUCTOR, 4),
            Unlock::Upgrade(ASSEMBLER, 4),
            Unlock::Upgrade(BLAST_FURNACE, 4),
            Unlock::Upgrade(DRONE_PORT, 4),
            Unlock::Upgrade(RESEARCH_CENTER, 4),
            Unlock::Upgrade(STORAGE, 4),
            Unlock::Upgrade(PUMPJACK, 1),
            Unlock::Upgrade(THUMPER, 2),
        ],
    },
    Tech {
        name: "Chip Fab",
        blurb: "The chip fab (1 MW, 4×4×3) etches wafers from silicon, acid and pure water (made in the chemical \
                plant) and builds AI accelerators: the rack part every datacenter, satellite and laser link uses.",
        ring: Ring::Gold,
        wedge: Wedge::Computing,
        deed: None,
        needs: &[ACIDS_AND_LUBRICANTS, PROCESSORS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 260,
        seconds: 50.0,
        unlocks: &[
            r(CHIP_FAB_BLOCK),
            Unlock::MachineRecipe(PURE_WATER_RECIPE),
            Unlock::MachineRecipe(WAFER_RECIPE),
            Unlock::MachineRecipe(ACCELERATOR_RECIPE),
        ],
    },
    Tech {
        name: "Data Network",
        blurb: "Fibre nodes join into data grids within 12 blocks; machines that make or use compute hang on the \
                nearest node within 5.",
        ring: Ring::Gold,
        wedge: Wedge::Computing,
        deed: None,
        needs: &[PROCESSORS, PLASTICS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 150,
        seconds: 40.0,
        unlocks: &[r(FIBRE_NODE)],
    },
    Tech {
        name: "Datacenters",
        blurb: "The AI datacenter (4×4×3) turns 3 MW and cooling water into 100 TF for the data grid; the cooling \
                tower (300 kW, 3×3×5) closes the water loop so one tower cools two datacenters.",
        ring: Ring::Gold,
        wedge: Wedge::Computing,
        deed: None,
        needs: &[CHIP_FAB, DATA_NETWORK, FLUID_HANDLING],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 300,
        seconds: 60.0,
        unlocks: &[r(DATACENTER), r(COOLING_TOWER)],
    },
    Tech {
        name: "AI Labs",
        blurb: "The AI lab (2×2×2, 100 kW) researches at twice a lab's speed, makes every 2nd unit free of packs, \
                and uses 20 TF of compute. It is the only lab that can research compute-only techs.",
        ring: Ring::Gold,
        wedge: Wedge::Computing,
        deed: None,
        needs: &[DATACENTERS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 200,
        seconds: 60.0,
        unlocks: &[r(AI_LAB)],
    },
    BONUS[0],
    BONUS[1],
    BONUS[2],
    Tech {
        name: "Optimizer",
        blurb: "The optimizer node (2×2×2, 150 kW) uses 20 TF of the data grid to make every machine within 16 \
                blocks work 25% faster. Machines take the best optimizer in range; they do not stack.",
        ring: Ring::Gold,
        wedge: Wedge::Computing,
        deed: None,
        needs: &[DATACENTERS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 250,
        seconds: 60.0,
        unlocks: &[r(OPTIMIZER)],
    },
    Tech {
        name: "Photonics",
        blurb: "Laser links: an emitter shoots a beam along its front to a receiver or data receiver up to 128 blocks \
                away; mirrors turn the beam round corners, glass lets it through.",
        ring: Ring::Gold,
        wedge: Wedge::Power,
        deed: None,
        needs: &[CHIP_FAB, DATA_NETWORK, BAUXITE_PROCESSING],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 150,
        seconds: 60.0,
        unlocks: &[r(LASER_EMITTER), r(LASER_RECEIVER), r(LASER_MIRROR), r(DATA_RECEIVER)],
    },
    Tech {
        name: "Drone Swarms",
        blurb: "The swarm hub (2×2×2, 100 kW) uses 30 TF of the data grid to let every drone port within 12 blocks \
                keep 50% more drones. Ports take the best hub in range; hubs do not stack.",
        ring: Ring::Gold,
        wedge: Wedge::RobotsDrones,
        deed: None,
        needs: &[DATACENTERS, CONSTRUCTION_DRONES, BAUXITE_PROCESSING],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 250,
        seconds: 60.0,
        unlocks: &[r(SWARM_HUB)],
    },
    Tech {
        name: "Rocketry",
        blurb:
            "Light frames, rocket engines, fuel tanks, avionics, rocket fuel and survey satellites. The launch site \
                (7×7×6, 2 MW) builds and launches rockets into orbit — the first launch opens the Sky ring.",
        ring: Ring::Gold,
        wedge: Wedge::Survey,
        deed: None,
        needs: &[ELECTROLYSIS, CHIP_FAB, SOLAR_POWER, BAUXITE_PROCESSING],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 300,
        seconds: 60.0,
        unlocks: &[
            r(LAUNCH_SITE),
            Unlock::MachineRecipe(ROCKET_PART_RECIPES[0]),
            Unlock::MachineRecipe(ROCKET_PART_RECIPES[1]),
            Unlock::MachineRecipe(ROCKET_PART_RECIPES[2]),
            Unlock::MachineRecipe(ROCKET_PART_RECIPES[3]),
            Unlock::MachineRecipe(ROCKET_FUEL_RECIPE),
            Unlock::MachineRecipe(ROCKET_RECIPE),
            Unlock::MachineRecipe(SURVEY_SATELLITE_RECIPE),
        ],
    },
    // --- Sky ring (72..76, opened by the first rocket launch) ---
    Tech {
        name: "Orbital Comms",
        blurb: "Comms satellites cover 1024 blocks around the ground below them: fibre nodes join into one data grid \
                and drone ports reach four times as far while a satellite is overhead.",
        ring: Ring::Sky,
        wedge: Wedge::Survey,
        deed: None,
        needs: &[ROCKETRY, DATA_NETWORK],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 250,
        seconds: 60.0,
        unlocks: &[
            Unlock::MachineRecipe(COMMS_SATELLITE_RECIPES[0]),
            Unlock::MachineRecipe(COMMS_SATELLITE_RECIPES[1]),
        ],
    },
    Tech {
        name: "Orbital Power",
        blurb: "Power satellites beam 200 kW each to a rectenna (5×5) under their 1024-block circle: up to 1 MW a \
                field, day and night, with no fuel.",
        ring: Ring::Sky,
        wedge: Wedge::Power,
        deed: None,
        needs: &[ROCKETRY, SOLAR_POWER],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 300,
        seconds: 60.0,
        unlocks: &[
            r(RECTENNA),
            Unlock::MachineRecipe(POWER_SATELLITE_RECIPES[0]),
            Unlock::MachineRecipe(POWER_SATELLITE_RECIPES[1]),
        ],
    },
    Tech {
        name: "Orbital Science",
        blurb: "Science satellites gather data from orbit. The ground station (3×3 dish) downlinks 40 TF of compute \
                per satellite overhead, up to 100 TF, with no datacenter.",
        ring: Ring::Sky,
        wedge: Wedge::Survey,
        deed: None,
        needs: &[ROCKETRY, DATA_NETWORK],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 300,
        seconds: 60.0,
        unlocks: &[
            r(GROUND_STATION),
            Unlock::MachineRecipe(SCIENCE_SATELLITE_RECIPES[0]),
            Unlock::MachineRecipe(SCIENCE_SATELLITE_RECIPES[1]),
        ],
    },
    Tech {
        name: "Drop Pods",
        blurb: "Ship cargo through the sky. A pod launcher (3×3×3, 500 kW) fires a 6-stack drop pod to the nearest \
                landing pad on the same colour channel, needing a comms satellite overhead.",
        ring: Ring::Sky,
        wedge: Wedge::Logistics,
        deed: None,
        needs: &[ORBITAL_COMMS],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 250,
        seconds: 60.0,
        unlocks: &[r(POD_LAUNCHER), r(LANDING_PAD), Unlock::MachineRecipe(DROP_POD_RECIPE)],
    },
];
