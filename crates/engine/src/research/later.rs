//! The open end of `techs::TECHS`: techs added after Milestone 12's first steps, so that `orbit.rs` stays closed and no saved
//! index moves. Saved index order: Long-Distance Power 82, Drop Pods 83, Settlements 84, Architecture 85, Forestry 86 (violet packs, after Violet Science 18), X-ray Goggles 87 (blue packs, after Hoists 51), LED Lamps 88 (blue packs, after Architecture 85 and Electronics 17), Pigments 89 (four packs, after Acids and Lubricants 45 and Architecture 85), Gas Wells 90 (four packs, after Oil Processing 42), Gas Processing 91 (four packs, after Gas Wells 90 and Acids and Lubricants 45), Flamethrower 92 (four packs, after Refining 43), Seismic Prospecting 93 (four packs, after Advanced Scanning 22 and Violet Science 18), Ore Scanning 94 (two packs, after Green Science 2).
//! Add rows at the bottom, never reorder (new Orbit techs go here too). Needs name earlier indices: Green Science 2, Fluid Handling 6, Masonry 7, Assembly 8, Steelmaking 9, Blue Science 10,
//! Violet Science 18, Mk4 Logistics 19, Advanced Scanning 22, Bauxite Processing 36, Oil Processing 42, Acids and Lubricants 45, Rocketry 78, Orbital Comms 79.

use crate::block::{
    FORESTER, GAS_PLANT, GAS_TURBINE, GAS_WELL, GLASS_KILN, LANDING_PAD, LED, LONG_POLE, POD_LAUNCHER, STEAM_REFORMER,
    STREET, THUMPER, TOWN_DEPOT, WELLHEAD_SEPARATOR,
};
use crate::item::{
    BLUE_PACK, FLAMETHROWER, GOLD_PACK, GREEN_PACK, PAINT_SPRAYER, RED_PACK, SCANNER, VIOLET_PACK, XRAY_GOGGLES,
};
use crate::recipes::{
    DECOR_FIRST, DECOR_ROWS, DROP_POD_RECIPE, DYE_FIRST, DYE_PLANT_ROWS, ETHANE_CRACK_RECIPE, GAS_PROCESS_RECIPE,
    GLASS_FIRST, GLASS_PRIMARY_COUNT, GLASS_RAW_FIRST, GLASS_ROWS, LONG_POLE_ROW, LPG_CRACK_RECIPE,
    POLYETHYLENE_RECIPE, REFORM_RECIPE,
};

use super::{r, Tech, Unlock};

/// Architecture unlocks: Glass Kiln, Paint Sprayer, non-concrete decor rows (9) and the 5 primary-pigment
/// stained glass batches (Glass, Sand, Quartz = 15 rows).
const DECOR_UNLOCKS: [Unlock; 2 + (DECOR_ROWS - 5) as usize + 3 * GLASS_PRIMARY_COUNT as usize] = {
    let mut out =
        [Unlock::MachineRecipe(DECOR_FIRST + 5); 2 + (DECOR_ROWS - 5) as usize + 3 * GLASS_PRIMARY_COUNT as usize];
    out[0] = r(GLASS_KILN);
    out[1] = Unlock::Recipe(PAINT_SPRAYER);
    let mut k = 2;
    let mut i = 5;
    while i < DECOR_ROWS as usize {
        out[k] = Unlock::MachineRecipe(DECOR_FIRST + i as u16);
        k += 1;
        i += 1;
    }
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

/// Pigments unlocks: the chemical plant's 11 mixed pigments plus the 11 mixed-pigment stained glass batches
/// (Glass, Sand, Quartz = 33 rows) in the Glass Kiln.
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

pub const LATER: [Tech; 13] = [
    Tech {
        name: "Long-Distance Power",
        blurb: "Lattice towers, 2×2 and 3 high, that link to each other 160 blocks apart (an ordinary pole only 16), \
                to carry power across the wilderness to far mines and oil wells.",
        needs: &[19, 36],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 150,
        seconds: 30.0,
        unlocks: &[r(LONG_POLE), Unlock::MachineRecipe(LONG_POLE_ROW)],
    },
    Tech {
        name: "Drop Pods",
        blurb:
            "Ship cargo through the sky. An assembler builds drop pods; a pod launcher (3×3×3, 500 kW) loads up to 6 \
                stacks from its belts into one and sends it up, and 20 seconds later it lands on a landing pad (3×3) \
                that belts empty. Both need a comms satellite overhead, and a launcher and a pad pair up when you tune \
                them to the same colour (six channels): a pod goes to the nearest pad on its channel.",
        needs: &[78, 79],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
        units: 250,
        seconds: 60.0,
        unlocks: &[r(POD_LAUNCHER), r(LANDING_PAD), Unlock::MachineRecipe(DROP_POD_RECIPE)],
    },
    Tech {
        name: "Settlements",
        blurb:
            "Found towns. A town depot (3×3, 2 high) placed at a fitting site founds one, of a kind that site decides: \
                a harbour on a sea shore, a river town on a river or lake bank, a highland workshop in the granite \
                highlands, a basalt forge in the basalt fields. Towns stand at least 300 blocks apart and away from \
                the spawn plains; while you hold a depot it says whether the spot fits. Belts fill it from any side.",
        needs: &[10],
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 100,
        seconds: 20.0,
        unlocks: &[r(TOWN_DEPOT)],
    },
    Tech {
        name: "Architecture",
        blurb:
            "Build for looks. The Paint Sprayer coats concrete, plaster, masonry, metal and roof surfaces in any \
                pigment (grind iron, copper, coal, quicklime or sulfur for the first five); the Glass Kiln melts glass, \
                sand or quartz with pigment into stained glass; smelters fire rock and slag into tiles and bricks; \
                assemblers make framed glass, metal panels, catwalks, railings and panel lights.",
        needs: &[7, 8],
        packs: &[RED_PACK, GREEN_PACK],
        units: 40,
        seconds: 10.0,
        unlocks: &DECOR_UNLOCKS,
    },
    Tech {
        name: "Forestry",
        blurb: "The forester: it keeps a patch of land in front of it as a wood lot, felling each tree, gathering its \
                saplings and planting them again, for a slow but endless supply of logs.",
        needs: &[18], // Violet Science
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 15,
        seconds: 20.0,
        unlocks: &[r(FORESTER)],
    },
    Tech {
        name: "X-ray Goggles",
        blurb: "Goggles for the Head slot that show the belts, pipes, poles and machines you built through the ground \
                around you, tinted, so a lost underpass or lift line can be found without digging. Open the inventory \
                and Shift-click them to put them on; a key switches the view on and off.",
        needs: &[51], // Hoists
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 120,
        seconds: 30.0,
        unlocks: &[Unlock::Recipe(XRAY_GOGGLES)],
    },
    Tech {
        name: "LED Lamps",
        blurb: "A lamp that lights twice the ground of a plain one and takes any colour. Right-click one to pick its \
                colour and one of three brightnesses. Also tall street lamps. No power needed.",
        needs: &[85, 17], // Architecture, Electronics (the silicon)
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
        units: 80,
        seconds: 20.0,
        unlocks: &[r(LED), r(STREET)],
    },
    Tech {
        name: "Pigments",
        blurb: "The chemical plant makes blue pigment from a copper ingot, a barrel of acid and a unit of water, and \
                mixes two pigments into ten more hues (orange, purple, cyan, lime, pink, sky, grey, brown, maroon and \
                navy) for the Paint Sprayer and the Glass Kiln.",
        needs: &[45, 85], // Acids and Lubricants (the acid), Architecture (the Paint Sprayer and Glass Kiln)
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 40.0,
        unlocks: &DYE_UNLOCKS,
    },
    Tech {
        name: "Gas Wells",
        blurb: "Tap deep natural gas shale fields (220+ blocks out under plains, lowlands and deserts). A gas well \
                (90 kW) fills empty barrels with raw gas; a gas plant (140 kW, water pipe) sweetens 4 raw gas barrels \
                into 2 methane, 1 ethane, 1 LPG and a sulfur.",
        needs: &[42], // Oil Processing (empty barrels & fluids)
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 140,
        seconds: 40.0,
        unlocks: &[r(GAS_WELL), r(GAS_PLANT), Unlock::MachineRecipe(GAS_PROCESS_RECIPE)],
    },
    Tech {
        name: "Gas Processing",
        blurb: "Turn natural gas into clean power, cheap hydrogen and plastics: a gas turbine gives 600 kW from \
                methane or LPG barrels, a steam reformer (100 kW) turns methane and water into 3 hydrogen barrels, \
                crackers split ethane or LPG into ethylene for polyethylene plastic, and a wellhead separator recovers \
                associated gas from any existing oil field.",
        needs: &[90, 45], // Gas Wells, Acids and Lubricants (lubricant, cracker & chemical plant)
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 160,
        seconds: 40.0,
        unlocks: &[
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
        blurb: "A hand liquid-fuel torch for clearing weeds and forests. Hold right-click for a steady flame that \
                clears ground plants right where you aim (like a weed torch, without spreading across meadows) and \
                sets grown trees ablaze so they burn down to charcoal. Runs on naphtha or diesel barrels from the \
                refinery or cracker; never harms placed blocks, machines or houses.",
        needs: &[43], // Refining (naphtha & diesel barrels from refinery or cracker)
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 30.0,
        unlocks: &[Unlock::Recipe(FLAMETHROWER)],
    },
    Tech {
        name: "Seismic Prospecting",
        blurb: "The Seismic Thumper (2×2×2, 40 kW): wire it to a power pole and right-click to scan scrubbable 64-block \
                vertical cross-sections and an interactive 3D subsurface cutaway down to bedrock — showing dense ore \
                bands, caves, aquifers and your own tunnels. Upgrade to Mk2 (violet kits) and Mk3 (gold kits).",
        needs: &[22, 18], // Advanced Scanning (Blue), Violet Science
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 30.0,
        unlocks: &[r(THUMPER)],
    },
    Tech {
        name: "Ore Scanning",
        blurb: "The hand-held Scanner: right-click to list every ore deposit within 48 blocks — what lies underground, \
                how deep, which direction and how large.",
        needs: &[2], // Green Science
        packs: &[RED_PACK, GREEN_PACK],
        units: 25,
        seconds: 10.0,
        unlocks: &[Unlock::Recipe(SCANNER)],
    },
];
