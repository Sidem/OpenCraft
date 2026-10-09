//! Hand recipes of the worn gear (`equipment.rs`), one per item; each is a tech's unlock (`research/personal.rs`).
//! To add one: a `pub const` here, and its name in `RECIPES`.

use crate::item::*;

use super::{Group, Recipe};

pub const HAULER_PACK_RECIPE: Recipe = Recipe {
    output: HAULER_PACK,
    group: Group::Tools,
    count: 1,
    inputs: &[(IRON_PLATE, 6), (STEEL_PLATE, 2), (SCREW, 10)],
    blurb: "Wear it on your back (open the inventory and Shift-click it): 27 more backpack slots.",
};

// Mk2 to Mk5 are the pack below plus `PACK_KITS` kits of the tier (`equipment::next_pack`; a test checks it). A worn
// pack is raised in place from the equipment panel with the same kits, so it never has to come off.
pub const HAULER_PACK_MK2_RECIPE: Recipe = Recipe {
    output: HAULER_PACK_MK2,
    group: Group::Tools,
    count: 1,
    inputs: &[(HAULER_PACK, 1), (GREEN_KIT, 4)],
    blurb: "A hauler pack with a reinforced frame: 18 more backpack slots, 45 in all. Made from a hauler pack, or \
            raise the worn one in the inventory.",
};

pub const HAULER_PACK_MK3_RECIPE: Recipe = Recipe {
    output: HAULER_PACK_MK3,
    group: Group::Tools,
    count: 1,
    inputs: &[(HAULER_PACK_MK2, 1), (BLUE_KIT, 4)],
    blurb: "18 more backpack slots, 63 in all. Made from a Mk2 pack, or raise the worn one in the inventory.",
};

pub const HAULER_PACK_MK4_RECIPE: Recipe = Recipe {
    output: HAULER_PACK_MK4,
    group: Group::Tools,
    count: 1,
    inputs: &[(HAULER_PACK_MK3, 1), (VIOLET_KIT, 4)],
    blurb: "18 more backpack slots, 81 in all. Made from a Mk3 pack, or raise the worn one in the inventory.",
};

pub const HAULER_PACK_MK5_RECIPE: Recipe = Recipe {
    output: HAULER_PACK_MK5,
    group: Group::Tools,
    count: 1,
    inputs: &[(HAULER_PACK_MK4, 1), (GOLD_KIT, 4)],
    blurb: "18 more backpack slots, 99 in all. Made from a Mk4 pack, or raise the worn one in the inventory.",
};

pub const SPRING_BOOTS_RECIPE: Recipe = Recipe {
    output: SPRING_BOOTS,
    group: Group::Tools,
    count: 1,
    inputs: &[(STEEL_PLATE, 4), (GEAR, 4), (IRON_ROD, 4)],
    blurb: "Boots on steel coils: you jump 2 blocks high instead of 1. One pair of boots at a time.",
};

pub const SERVO_BOOTS_RECIPE: Recipe = Recipe {
    output: SERVO_BOOTS,
    group: Group::Tools,
    count: 1,
    inputs: &[(STEEL_PLATE, 2), (SERVO, 2), (CIRCUIT, 2)],
    blurb: "Boots with servo joints: you walk and sprint 15% faster. One pair of boots at a time.",
};

pub const EXO_FRAME_RECIPE: Recipe = Recipe {
    output: EXO_FRAME,
    group: Group::Tools,
    count: 1,
    inputs: &[(STEEL_BEAM, 4), (ACTUATOR, 2), (CIRCUIT, 3)],
    blurb: "A steel frame worn over the chest: you sprint 10% faster.",
};

pub const MINING_RIG_RECIPE: Recipe = Recipe {
    output: MINING_RIG,
    group: Group::Tools,
    count: 1,
    inputs: &[(STEEL_PLATE, 3), (MOTOR, 2), (GEAR, 3)],
    blurb: "A tool belt with a motor-driven drill: you break blocks by hand 50% faster.",
};
