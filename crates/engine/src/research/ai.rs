//! Milestone 11's techs that build on AI Research and the bonus techs (appended after `bonus.rs`'s in `techs::TECHS`,
//! so the saved index order is: Optimizer 66; add rows at the bottom, never reorder).

use crate::block::OPTIMIZER;
use crate::item::{BLUE_PACK, GOLD_PACK, GREEN_PACK, RED_PACK, VIOLET_PACK};

use super::{r, Tech};

pub const AI: [Tech; 1] = [Tech {
    name: "Optimizer",
    blurb:
        "The optimizer node (2×2×2, 150 kW) uses 20 TF of the data grid to make every machine within 16 blocks work \
            25% faster. Machines take the best optimizer in range; they do not stack.",
    needs: &[62],
    packs: &[RED_PACK, GREEN_PACK, BLUE_PACK, VIOLET_PACK, GOLD_PACK],
    units: 250,
    seconds: 60.0,
    unlocks: &[r(OPTIMIZER)],
}];
