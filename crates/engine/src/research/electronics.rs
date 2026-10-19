//! Techs 17–29 of `techs::TECHS`, Milestones 7 and 8: electronics, violet science, Mk4, solar power, logic and the
//! drones. Joined straight after the main table (`techs.rs`), so no index moved. Closed: new techs go in `later.rs`.

use crate::block::{
    ACCUMULATOR, ARC_FURNACE, ASSEMBLER, BELT, BLAST_FURNACE, CONSTRUCTOR, DRONE_PORT, LAB, MINER, POLE, SENSOR,
    SMELTER, SOLAR_PANEL, UNDERPASS_IN,
};
use crate::item::{BLUE_PACK, GREEN_PACK, RED_PACK, SCANNER_MK2, VIOLET_PACK};
use crate::recipes::{ACCUMULATOR_ROW, DRONE_RECIPES, ELECTRONICS_RECIPES, SOLAR_PANEL_ROW, VIOLET_RECIPES};

use super::{r, Tech, Unlock};

pub(super) const ELECTRONICS: [Tech; 13] = [
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
        blurb: "Violet kits, made in an assembler, and Mk4 belts (eight times a Mk1) and underpasses, and substations that link 32 \
                blocks and power machines within 16.",
        needs: &[18],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 100,
        seconds: 30.0,
        unlocks: &[
            Unlock::MachineRecipe(VIOLET_RECIPES[1]),
            Unlock::Upgrade(BELT, 3),
            Unlock::Upgrade(UNDERPASS_IN, 3),
            Unlock::Upgrade(POLE, 3),
        ],
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
        unlocks: &[
            r(SOLAR_PANEL),
            r(ACCUMULATOR),
            Unlock::MachineRecipe(SOLAR_PANEL_ROW),
            Unlock::MachineRecipe(ACCUMULATOR_ROW),
        ],
    },
    Tech {
        name: "Advanced Scanning",
        blurb: "The Scanner Mk2, built from a scanner, circuits and steel: 96 blocks of range, an ore filter, ore \
                units and mining time per deposit, and a pointer to the nearest match.",
        needs: &[18],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 60,
        seconds: 25.0,
        unlocks: &[Unlock::Recipe(SCANNER_MK2)],
    },
    Tech {
        name: "Logic",
        blurb: "Sensors read the box, silo or belt behind them and switch the machine in front on or off: stop a \
                generator when the accumulators are full, feed a smelter only while its box has room.",
        needs: &[18],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 80,
        seconds: 25.0,
        unlocks: &[r(SENSOR)],
    },
    Tech {
        name: "Processors",
        blurb: "Assemblers solder processors from circuits, silicon and steel: the brains of everything that thinks \
                for itself.",
        needs: &[18],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 100,
        seconds: 30.0,
        unlocks: &[Unlock::MachineRecipe(DRONE_RECIPES[0])],
    },
    Tech {
        name: "Robotics",
        blurb: "Servos (a motor, circuits and gears) and the actuators built from them: the muscles of a machine \
                that moves. Assemblers only.",
        needs: &[20, 24],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 30.0,
        unlocks: &[Unlock::MachineRecipe(DRONE_RECIPES[1]), Unlock::MachineRecipe(DRONE_RECIPES[2])],
    },
    Tech {
        name: "Drone Power",
        blurb: "The drone cell: a motor, circuits and steel plates packed into a flight-ready \
                power pack. Slow to assemble.",
        needs: &[25],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 160,
        seconds: 35.0,
        unlocks: &[Unlock::MachineRecipe(DRONE_RECIPES[3])],
    },
    Tech {
        name: "Navigation",
        blurb: "The guidance module: a processor, circuits and copper wire that let a machine find a spot in the \
                air and hold it.",
        needs: &[24, 25],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 180,
        seconds: 35.0,
        unlocks: &[Unlock::MachineRecipe(DRONE_RECIPES[4])],
    },
    Tech {
        name: "Construction Drones",
        blurb: "The drone: two actuators, a drone cell and a guidance module, 30 seconds in an assembler, and the \
                drone port, a 3×3 pad that keeps four and sends them out to build your ghosts and tear down what \
                you mark, working from storage boxes beside it.",
        needs: &[26, 27],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 240,
        seconds: 40.0,
        unlocks: &[Unlock::MachineRecipe(DRONE_RECIPES[5]), r(DRONE_PORT)],
    },
    Tech {
        name: "Swarm Logistics",
        blurb: "Kits upgrade drone ports in place: Mk2 keeps 8 drones and reaches 48 blocks, Mk3 12 and 64, Mk4 16 \
                and 96.",
        needs: &[28],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 200,
        seconds: 40.0,
        unlocks: &[Unlock::Upgrade(DRONE_PORT, 1), Unlock::Upgrade(DRONE_PORT, 2), Unlock::Upgrade(DRONE_PORT, 3)],
    },
];
