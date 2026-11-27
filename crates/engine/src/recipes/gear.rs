//! Hand recipes of the worn gear (`equipment.rs`), one per item; each is a tech's unlock (`research/personal.rs`) but
//! the headlamp's and the rope's.
//! To add one: a `pub const` here, and its name in `RECIPES`.

use crate::block::{LAMP, LEAVES, STONE};
use crate::item::*;

use super::{b, Group, Recipe};

pub const HAULER_PACK_RECIPE: Recipe = Recipe {
    output: HAULER_PACK,
    group: Group::Tools,
    count: 1,
    inputs: &[(IRON_PLATE, 6), (STEEL_PLATE, 2), (SCREW, 10)],
    blurb: "Wear it on your back (open the inventory and Shift-click it): 27 more backpack slots.",
};

// Mk2 to Mk5 are the pack below plus `PACK_KITS` kits of the tier (`equipment::next_tier`; a test checks it). A worn
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
    blurb: "Boots with servo joints: you step up a whole block as you walk, and walk 15% faster. One pair at a time.",
};

pub const EXO_FRAME_RECIPE: Recipe = Recipe {
    output: EXO_FRAME,
    group: Group::Tools,
    count: 1,
    inputs: &[(STEEL_BEAM, 4), (ACTUATOR, 2), (CIRCUIT, 3)],
    blurb: "A steel frame worn over the chest: your hands reach 8 blocks instead of 5.",
};

pub const WORK_GLOVES_RECIPE: Recipe = Recipe {
    output: WORK_GLOVES,
    group: Group::Tools,
    count: 1,
    inputs: &[(IRON_PLATE, 4), (SCREW, 6), (COPPER_WIRE, 4)],
    blurb: "Work gloves worn in the Gloves slot: craft by hand 25% faster. Raise with kits like the hauler pack.",
};

pub const WORK_GLOVES_MK2_RECIPE: Recipe = Recipe {
    output: WORK_GLOVES_MK2,
    group: Group::Tools,
    count: 1,
    inputs: &[(WORK_GLOVES, 1), (GREEN_KIT, 4)],
    blurb: "Work gloves with geared linkages: craft by hand 50% faster.",
};

pub const WORK_GLOVES_MK3_RECIPE: Recipe = Recipe {
    output: WORK_GLOVES_MK3,
    group: Group::Tools,
    count: 1,
    inputs: &[(WORK_GLOVES_MK2, 1), (BLUE_KIT, 4)],
    blurb: "Powered work gloves: craft by hand 75% faster.",
};

pub const WORK_GLOVES_MK4_RECIPE: Recipe = Recipe {
    output: WORK_GLOVES_MK4,
    group: Group::Tools,
    count: 1,
    inputs: &[(WORK_GLOVES_MK3, 1), (VIOLET_KIT, 4)],
    blurb: "Precision work gloves: craft by hand 100% faster.",
};

pub const CRAFT_GLOVES_RECIPE: Recipe = Recipe {
    output: CRAFT_GLOVES,
    group: Group::Tools,
    count: 1,
    inputs: &[(WORK_GLOVES_MK4, 1), (GOLD_KIT, 4)],
    blurb: "Gloves with servo fingers, worn in the Gloves slot: craft by hand 125% faster.",
};

pub const XRAY_GOGGLES_RECIPE: Recipe = Recipe {
    output: XRAY_GOGGLES,
    group: Group::Tools,
    count: 1,
    inputs: &[(STEEL_PLATE, 2), (CIRCUIT, 4), (COPPER_WIRE, 6), (SILICON, 2)],
    blurb: "Goggles that see the machines you built through solid ground, worn in the Head slot. They light the way.",
};

// Needs no tech, like the lamp it is made from (red era: glass, plate and wire).
pub const HEADLAMP_RECIPE: Recipe = Recipe {
    output: HEADLAMP,
    group: Group::Tools,
    count: 1,
    inputs: &[(b(LAMP), 1), (IRON_PLATE, 1), (COPPER_WIRE, 2)],
    blurb: "A lamp on a strap, worn in the Head slot: it lights the way like a lamp in hand, both hands free.",
};

// Needs no tech: leaves twisted round sticks. One coil hangs `action::PER_COIL` blocks.
pub const ROPE_RECIPE: Recipe = Recipe {
    output: ROPE_ITEM,
    group: Group::Tools,
    count: 1,
    inputs: &[(b(LEAVES), 4), (STICK, 2)],
    blurb: "A coil of rope: hang it from the side of a ledge and it unrolls 8 blocks down. Climb it like a ladder.",
};

pub const LINE_HOOK_RECIPE: Recipe = Recipe {
    output: LINE_HOOK,
    group: Group::Tools,
    count: 1,
    inputs: &[(STEEL_PLATE, 2), (GEAR, 2), (IRON_ROD, 2), (COPPER_WIRE, 4)],
    blurb: "A grappling hook for the left hand (J): G pulls you to a block, or clips onto a power wire and rides it \
            from pole to pole. Jump lets go.",
};

// The Tool Belt replaced the nine hand tools (`tool_belt/`). Mk2 and Mk3 are the belt below plus plates (as the iron
// and steel tools took), Mk4 and Mk5 plus kits (`tool_belt::STEPS`; a test checks they match).
pub const TOOL_BELT_RECIPE: Recipe = Recipe {
    output: TOOL_BELT,
    group: Group::Tools,
    count: 1,
    inputs: &[(b(STONE), 6), (STICK, 4)],
    blurb: "Wear it in the Tool belt slot (Shift-click it): stone, wood and soil break twice as fast by hand, with no \
            tool held, and it never wears out. Raise it in the inventory.",
};

pub const TOOL_BELT_MK2_RECIPE: Recipe = Recipe {
    output: TOOL_BELT_MK2,
    group: Group::Tools,
    count: 1,
    inputs: &[(TOOL_BELT, 1), (IRON_PLATE, 6)],
    blurb: "Iron heads: 4× as fast, and 4 ore kept from each ore block instead of 3.",
};

pub const TOOL_BELT_MK3_RECIPE: Recipe = Recipe {
    output: TOOL_BELT_MK3,
    group: Group::Tools,
    count: 1,
    inputs: &[(TOOL_BELT_MK2, 1), (STEEL_PLATE, 6)],
    blurb: "Steel heads: 6×, 5 ore a block. Crouch while mining to dig a 1×2 tunnel.",
};

pub const TOOL_BELT_MK4_RECIPE: Recipe = Recipe {
    output: TOOL_BELT_MK4,
    group: Group::Tools,
    count: 1,
    inputs: &[(TOOL_BELT_MK3, 1), (VIOLET_KIT, 4)],
    blurb: "A rock drill: 8×. Crouch while mining to take the 3×3 face around the block, only blocks of its kind.",
};

pub const TOOL_BELT_MK5_RECIPE: Recipe = Recipe {
    output: TOOL_BELT_MK5,
    group: Group::Tools,
    count: 1,
    inputs: &[(TOOL_BELT_MK4, 1), (GOLD_KIT, 4)],
    blurb: "A laser cutter: 10×. Crouch while mining to cut up to 32 joined blocks of the same kind.",
};
