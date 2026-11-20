//! The open end of `techs::TECHS`: techs added after Milestone 12's first steps, so that `orbit.rs` stays closed and no saved
//! index moves. Saved index order: Long-Distance Power 82, Drop Pods 83, Settlements 84, Architecture 85, Forestry 86 (violet packs, after Violet Science 18), X-ray Goggles 87 (blue packs, after Hoists 51), LED Lamps 88 (blue packs, after Architecture 85 and Electronics 17), Pigments 89 (four packs, after Acids and Lubricants 45 and Architecture 85).
//! Add rows at the bottom, never reorder (new Orbit techs go here too). Needs name earlier indices: Masonry 7, Assembly 8, Blue Science 10,
//! Mk4 Logistics 19, Bauxite Processing 36, Rocketry 78, Orbital Comms 79.

use crate::block::{FORESTER, LANDING_PAD, LED, LONG_POLE, POD_LAUNCHER, STREET, TOWN_DEPOT};
use crate::item::{BLUE_PACK, GOLD_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK, XRAY_GOGGLES};
use crate::recipes::{DECOR_FIRST, DECOR_ROWS, DROP_POD_RECIPE, DYE_FIRST, DYE_ROWS, LONG_POLE_ROW};

use super::{r, Tech, Unlock};

/// Every machine row of the decor set.
const DECOR_UNLOCKS: [Unlock; DECOR_ROWS as usize] = {
    let mut out = [Unlock::MachineRecipe(DECOR_FIRST); DECOR_ROWS as usize];
    let mut i = 0;
    while i < out.len() {
        out[i] = Unlock::MachineRecipe(DECOR_FIRST + i as u16);
        i += 1;
    }
    out
};

/// Every machine row of the Pigments tech: the plant's blue and mixes, then the assembler's tints.
const DYE_UNLOCKS: [Unlock; DYE_ROWS as usize] = {
    let mut out = [Unlock::MachineRecipe(DYE_FIRST); DYE_ROWS as usize];
    let mut i = 0;
    while i < out.len() {
        out[i] = Unlock::MachineRecipe(DYE_FIRST + i as u16);
        i += 1;
    }
    out
};

pub const LATER: [Tech; 8] = [
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
            "Build for looks. Assemblers tint concrete red, green, black, white or yellow with a pinch of the ground \
                pigment (grind an iron or copper ingot, coal, quicklime or sulfur by hand); smelters fire basalt, granite, sandstone and slag into tiles \
                and bricks; assemblers make framed glass, metal panels, catwalks, railings and panel lights.",
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
                mixes two pigments into a new hue (orange, purple, cyan, lime, pink, sky, grey, brown, maroon and \
                navy). Assemblers tint concrete in all eleven.",
        needs: &[45, 85], // Acids and Lubricants (the acid), Architecture (the concrete tints)
        packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK],
        units: 120,
        seconds: 40.0,
        unlocks: &DYE_UNLOCKS,
    },
];
