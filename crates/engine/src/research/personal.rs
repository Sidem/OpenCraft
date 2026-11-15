//! The techs of what a player carries and wears, as data (appended to the end of `techs::TECHS`, so the order here
//! is the saved index order: add rows at the bottom, never reorder).

use crate::item::{
    BLUE_PACK, EXO_FRAME, GREEN_PACK, HAULER_PACK, HAULER_PACK_MK2, JETPACK, MINING_RIG, PERSONAL_DRONE, PLANNER,
    RED_PACK, SERVO_BOOTS, SPRING_BOOTS, VIOLET_PACK,
};

use super::{Tech, Unlock};

pub const PERSONAL: [Tech; 6] = [
    Tech {
        name: "Jetpack",
        blurb: "Four steel plates, two motors and two circuits: a coal-burning pack, worn in its own slot, that \
                lifts you while you hold jump. Ten seconds of thrust a coal.",
        needs: &[18, 22],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 30.0,
        unlocks: &[Unlock::Recipe(JETPACK)],
    },
    Tech {
        name: "Personal Drone",
        blurb: "A drone and two circuits make a companion that fetches what you ask for from the storage boxes \
                near you.",
        needs: &[28],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 160,
        seconds: 35.0,
        unlocks: &[Unlock::Recipe(PERSONAL_DRONE)],
    },
    Tech {
        name: "Earthworks",
        blurb: "The planner marks an area to dig, fill or flatten and shows what that would move. Drone ports do the \
                work: they dig into the storage boxes beside the pad and fill from them.",
        needs: &[28],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 35.0,
        unlocks: &[Unlock::Recipe(PLANNER)],
    },
    Tech {
        name: "Hauler Gear",
        blurb: "Equipment slots: wear a hauler pack on your back for 27 more backpack slots. Open the inventory and \
                Shift-click gear to put it on. Four green kits raise the pack to Mk2: 18 more slots, 45 in all.",
        needs: &[9],
        packs: &[RED_PACK, GREEN_PACK],
        units: 50,
        seconds: 10.0,
        unlocks: &[Unlock::Recipe(HAULER_PACK), Unlock::Recipe(HAULER_PACK_MK2)],
    },
    Tech {
        name: "Field Gear",
        blurb: "Spring boots to jump 2 blocks high, and a mining rig for your tool belt that breaks blocks by hand \
                50% faster.",
        needs: &[13],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 60,
        seconds: 20.0,
        unlocks: &[Unlock::Recipe(SPRING_BOOTS), Unlock::Recipe(MINING_RIG)],
    },
    Tech {
        name: "Exosuit",
        blurb: "Powered gear built from servos and actuators: servo boots (15% faster) and an exo frame (sprint 10% \
                faster).",
        needs: &[25, 34],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 30.0,
        unlocks: &[Unlock::Recipe(SERVO_BOOTS), Unlock::Recipe(EXO_FRAME)],
    },
];
// The hauler packs Mk3 to Mk5 are techs of their own in `ai.rs` (the last table, so indices stay put).
