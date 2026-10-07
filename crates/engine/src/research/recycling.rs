//! The Recycling tech, as data (appended after `chemistry.rs`'s in `techs::TECHS`, so its index is 55: add rows at the
//! bottom, never reorder). It follows Blue Science (index 10).

use crate::block::RECYCLER;
use crate::item::{BLUE_PACK, GREEN_PACK, RED_PACK};

use super::{r, Tech};

pub const RECYCLING: [Tech; 1] = [Tech {
    name: "Recycling",
    blurb:
        "The recycler (90 kW, 2×2×2) destroys any item you feed it, from six hatches, and pays recycling coins out of \
            two: a raw item 1, and every step of processing doubles it (an ingot 2, a plate 8). Slag and tailings, and \
            what is crushed from them, pay only 1. Coins stack to 1024; one day they buy cosmetics.",
    needs: &[10],
    packs: &[RED_PACK, GREEN_PACK, BLUE_PACK],
    units: 60,
    seconds: 20.0,
    unlocks: &[r(RECYCLER)],
}];
