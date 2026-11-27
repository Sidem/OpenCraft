//! Red (deeds + starter lab techs) and Green rings of [`super::TECHS`].

use crate::block::{
    ASSEMBLER, BELT, BELT_EJECTOR, BLAST_FURNACE, CONSTRUCTOR, FILTER, FORESTER, GENERATOR, GLASS_KILN, LAB, LIFT,
    MERGER, MINER, OUTLET, PIPE, POLE, PUMP, QUARRY, SMELTER, SPLITTER, STORAGE, UNDERPASS_IN, UPGRADER, WATER_WHEEL,
};
use crate::item::{
    GREEN_KIT, GREEN_PACK, HAULER_PACK, HAULER_PACK_MK2, PAINT_SPRAYER, RED_PACK, SCANNER, WORK_GLOVES, WORK_GLOVES_MK2,
};
use crate::recipes::{
    ASSEMBLY_RECIPES, BRICK_RECIPE, DECOR_FIRST, GEAR_RECIPE, GLASS_FIRST, GLASS_PRIMARY_COUNT, GLASS_RAW_FIRST,
    GLASS_ROWS, QUICKLIME_RECIPE, STEEL_RECIPES, UNDERPASS_ROW,
};

use super::ids::*;
use super::rings::{Deed, Ring, Wedge, BELT_DEED_TARGET, CLIMB_DEED_STEPS};
use super::{r, Feature, Tech, Unlock};

/// Glass unlocks: Glass Kiln, Framed Glass, and the 4 primary-pigment stained glass batches (Red, Green, Black, White;
/// Yellow needs Sulfur and moved to Pigments).
const GLASS_UNLOCKS: [Unlock; 2 + 3 * GLASS_PRIMARY_COUNT as usize] = {
    let mut out = [Unlock::MachineRecipe(GLASS_FIRST); 2 + 3 * GLASS_PRIMARY_COUNT as usize];
    out[0] = r(GLASS_KILN);
    out[1] = Unlock::MachineRecipe(DECOR_FIRST + 9); // Framed Glass
    let mut k = 2;
    let mut j = 0;
    while j < GLASS_PRIMARY_COUNT as usize {
        out[k] = Unlock::MachineRecipe(GLASS_FIRST + j as u16);
        out[k + 1] = Unlock::MachineRecipe(GLASS_RAW_FIRST + j as u16);
        out[k + 2] = Unlock::MachineRecipe(GLASS_RAW_FIRST + GLASS_ROWS + j as u16);
        k += 3;
        j += 1;
    }
    out
};

pub(super) const RED_GREEN: [Tech; 17] = [
    // --- Red ring (deeds, 0..2) ---
    Tech {
        name: "Belt Routing",
        blurb: "Deed: lay 40 belts. Unlocks splitters, filters and mergers to route items across lines.",
        ring: Ring::Red,
        wedge: Wedge::Logistics,
        deed: Some(Deed::LayBelts(BELT_DEED_TARGET)),
        needs: &[],
        packs: &[],
        units: 1,
        seconds: 1.0,
        unlocks: &[r(SPLITTER), r(FILTER), r(MERGER)],
    },
    Tech {
        name: "Belt Lifts",
        blurb: "Deed: run a belt line up 3 steps. Unlocks vertical lifts to carry items straight up cliffs.",
        ring: Ring::Red,
        wedge: Wedge::Logistics,
        deed: Some(Deed::ClimbBelt(CLIMB_DEED_STEPS)),
        needs: &[],
        packs: &[],
        units: 1,
        seconds: 1.0,
        unlocks: &[r(LIFT)],
    },
    // --- Red starter lab techs & Green ring (2..17; indices stay fixed for saves) ---
    Tech {
        name: "Underpasses",
        blurb: "Underpasses carry a belt under up to 4 blocks of anything. Drag a belt line over an obstacle and \
                it dives under by itself, if you carry underpasses. Belt ejectors end a belt by dropping its items \
                on the ground.",
        ring: Ring::Red,
        wedge: Wedge::Logistics,
        deed: None,
        needs: &[],
        packs: &[RED_PACK],
        units: 15,
        seconds: 10.0,
        unlocks: &[r(UNDERPASS_IN), r(BELT_EJECTOR), Unlock::MachineRecipe(UNDERPASS_ROW)],
    },
    Tech {
        name: "Mechanics",
        blurb: "Gears and green kits. Hold kits and right-click a machine or drag along a belt line to make it Mk2: \
                twice as fast, miners keep 75% instead of 60%, boxes hold 36 stacks, generators give 100 kW, poles \
                reach further and underpasses pass under 6 blocks.",
        ring: Ring::Green,
        wedge: Wedge::Workshop,
        deed: None,
        needs: &[],
        packs: &[RED_PACK, GREEN_PACK],
        units: 35,
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
            Unlock::Upgrade(FORESTER, 1),
            Unlock::Upgrade(LAB, 1),
            Unlock::Upgrade(GENERATOR, 1),
            Unlock::Upgrade(BELT, 1),
            Unlock::Upgrade(UNDERPASS_IN, 1),
        ],
    },
    Tech {
        name: "Fluid Handling",
        blurb: "Pumps lift water out of ponds and pits; pipes carry it to outlets that pour it out elsewhere.",
        ring: Ring::Red,
        wedge: Wedge::Power,
        deed: None,
        needs: &[],
        packs: &[RED_PACK],
        units: 15,
        seconds: 10.0,
        unlocks: &[r(PUMP), r(PIPE), r(OUTLET)],
    },
    Tech {
        name: "Masonry",
        blurb: "Smelters fire stone into bricks and limestone into quicklime: the blast furnace's bricks, and the \
                quicklime that steel and concrete need.",
        ring: Ring::Red,
        wedge: Wedge::Building,
        deed: None,
        needs: &[],
        packs: &[RED_PACK],
        units: 15,
        seconds: 10.0,
        unlocks: &[Unlock::MachineRecipe(BRICK_RECIPE), Unlock::MachineRecipe(QUICKLIME_RECIPE)],
    },
    Tech {
        name: "Assembly",
        blurb: "The assembler (2×2×2) puts parts together: motors, concrete, and packs and kits by the beltful. \
                Also unlocks the upgrader, ghost mode (B) and blueprints (Z, Enter, L) to plan and copy layouts.",
        ring: Ring::Green,
        wedge: Wedge::Workshop,
        deed: None,
        needs: &[MECHANICS, MASONRY],
        packs: &[RED_PACK, GREEN_PACK],
        units: 40,
        seconds: 10.0,
        unlocks: &[
            r(ASSEMBLER),
            r(UPGRADER),
            Unlock::Feature(Feature::GhostMode),
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
        ring: Ring::Green,
        wedge: Wedge::OresMetals,
        deed: None,
        needs: &[ASSEMBLY],
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
        name: "Hydropower",
        blurb: "The water wheel (3×1×3) turns falling water on one side into power: up to 10 kW for every falling \
                water block along that side (30 kW max), with no fuel and day and night.",
        ring: Ring::Green,
        wedge: Wedge::Power,
        deed: None,
        needs: &[MECHANICS],
        packs: &[RED_PACK, GREEN_PACK],
        units: 40,
        seconds: 15.0,
        unlocks: &[r(WATER_WHEEL)],
    },
    Tech {
        name: "Ore Scanning",
        blurb: "The hand-held Scanner: in your left hand it lists every ore deposit within 48 blocks — what lies \
                underground, how deep, which direction and how large.",
        ring: Ring::Red,
        wedge: Wedge::Survey,
        deed: None,
        needs: &[],
        packs: &[RED_PACK],
        units: 20,
        seconds: 10.0,
        unlocks: &[Unlock::Recipe(SCANNER)],
    },
    Tech {
        name: "Hauler Gear",
        blurb: "Equipment slots: wear a hauler pack on your back for 27 more backpack slots. Open the inventory and \
                Shift-click gear to put it on. Four green kits raise the pack to Mk2: 45 slots in all.",
        ring: Ring::Green,
        wedge: Wedge::YourGear,
        deed: None,
        needs: &[STEELMAKING, MECHANICS],
        packs: &[RED_PACK, GREEN_PACK],
        units: 50,
        seconds: 10.0,
        unlocks: &[Unlock::Recipe(HAULER_PACK), Unlock::Recipe(HAULER_PACK_MK2)],
    },
    Tech {
        name: "Work Gloves",
        blurb: "Wear work gloves in the Gloves slot to craft by hand 25% faster, and raise them with kits just like \
                the hauler pack: four green kits make Mk2 (+50%).",
        ring: Ring::Green,
        wedge: Wedge::YourGear,
        deed: None,
        needs: &[MECHANICS],
        packs: &[RED_PACK, GREEN_PACK],
        units: 40,
        seconds: 10.0,
        unlocks: &[Unlock::Recipe(WORK_GLOVES), Unlock::Recipe(WORK_GLOVES_MK2)],
    },
    Tech {
        name: "Forestry",
        blurb: "The forester: it keeps a patch of land in front of it as a wood lot, felling each tree, gathering its \
                saplings and planting them again, for a slow but endless supply of logs.",
        ring: Ring::Green,
        wedge: Wedge::TownsLife,
        deed: None,
        needs: &[],
        packs: &[RED_PACK, GREEN_PACK],
        units: 35,
        seconds: 15.0,
        unlocks: &[r(FORESTER)],
    },
    Tech {
        name: "Stone Styles",
        blurb: "Smelters fire basalt, granite, sandstone and blast-furnace slag into architectural tiles and bricks.",
        ring: Ring::Green,
        wedge: Wedge::Building,
        deed: None,
        needs: &[STEELMAKING],
        packs: &[RED_PACK, GREEN_PACK],
        units: 25,
        seconds: 10.0,
        unlocks: &[
            Unlock::MachineRecipe(DECOR_FIRST + 5),
            Unlock::MachineRecipe(DECOR_FIRST + 6),
            Unlock::MachineRecipe(DECOR_FIRST + 7),
            Unlock::MachineRecipe(DECOR_FIRST + 8),
        ],
    },
    Tech {
        name: "Glass",
        blurb: "The Glass Kiln melts glass, sand or quartz with red, green, black or white ground pigment into \
                stained glass; assemblers build framed glass.",
        ring: Ring::Green,
        wedge: Wedge::Building,
        deed: None,
        needs: &[MASONRY, ASSEMBLY],
        packs: &[RED_PACK, GREEN_PACK],
        units: 30,
        seconds: 10.0,
        unlocks: &GLASS_UNLOCKS,
    },
    Tech {
        name: "Metalwork",
        blurb: "Assemblers press metal panels, catwalks, railings and panel lights for industrial buildings.",
        ring: Ring::Green,
        wedge: Wedge::Building,
        deed: None,
        needs: &[ASSEMBLY],
        packs: &[RED_PACK, GREEN_PACK],
        units: 25,
        seconds: 10.0,
        unlocks: &[
            Unlock::MachineRecipe(DECOR_FIRST + 10),
            Unlock::MachineRecipe(DECOR_FIRST + 11),
            Unlock::MachineRecipe(DECOR_FIRST + 12),
            Unlock::MachineRecipe(DECOR_FIRST + 13),
        ],
    },
    Tech {
        name: "Paint",
        blurb: "The Paint Sprayer coats concrete, plaster, masonry, metal and roof surfaces in any pigment you carry \
                (24 surfaces per pigment).",
        ring: Ring::Green,
        wedge: Wedge::Building,
        deed: None,
        needs: &[MECHANICS],
        packs: &[RED_PACK, GREEN_PACK],
        units: 25,
        seconds: 10.0,
        unlocks: &[Unlock::Recipe(PAINT_SPRAYER)],
    },
];
