//! The hand recipes of Milestone 11's compute machines, listed in `RECIPES`: the chip fab (the Wafers tech:
//! `factory/process/fab.rs`) and the fibre node (Data Network: `factory/fibre.rs`).

use crate::block::*;
use crate::item::*;

use super::{b, Group, Recipe};

pub const CHIP_FAB_RECIPE: Recipe = Recipe {
    output: b(CHIP_FAB),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 60), (STEEL_BEAM, 24), (b(GLASS), 24), (PROCESSOR, 16), (CIRCUIT, 24)],
    blurb:
        "A clean room, 4×4×3 (R turns it before you place it). Wafers: 2 silicon, an acid canister and a pure water \
            canister make a wafer in 8 seconds and give both canisters back as empties (violet hatches on the right). \
            AI accelerators: 2 wafers, 2 processors and a plastic, 20 seconds. Belts bring everything in at the back \
            hatches and take the product from the front; it takes whatever its recipes use. Needs 1 MW.",
};

pub const DATACENTER_RECIPE: Recipe = Recipe {
    output: b(DATACENTER),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_BEAM, 40), (b(CONCRETE), 20), (AI_ACCELERATOR, 16), (COPPER_WIRE, 32)],
    blurb: "An AI datacenter, 4×4×3 (R turns it before you place it). Needs 3 MW all the time and gives the data grid \
            100 TF when it has it: put a fibre node within 5 blocks. It runs hot: pipe water from a pump to its blue \
            inlet on the left, or it shuts down after 20 seconds and gives nothing until it has cooled.",
};

pub const COOLING_TOWER_RECIPE: Recipe = Recipe {
    output: b(COOLING_TOWER),
    group: Group::Production,
    count: 1,
    inputs: &[(b(CONCRETE), 30), (b(PIPE), 12), (MOTOR, 4), (STEEL_PLATE, 12)],
    blurb: "A fan tower, 3×3×5 (R turns it before you place it), 300 kW. Pipe its blue inlet on the left to the same pipes \
            as a datacenter's inlet and the datacenter takes its coolant through the tower: the water comes back cool and \
            only one unit in twenty is lost, so a pump keeps up easily. One tower cools two datacenters. The tower needs a \
            pump on its pipes for that trickle of makeup water.",
};

pub const AI_LAB_RECIPE: Recipe = Recipe {
    output: b(AI_LAB),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 20), (AI_ACCELERATOR, 6), (PROCESSOR, 10), (CIRCUIT, 20), (b(GLASS), 8)],
    blurb: "A research lab for the data grid, 2×2×2. Like a research center it takes every science pack on every side, \
            at twice a lab's speed, and every 2nd unit costs no packs. While it works it uses 20 TF: put a fibre node \
            within 5 blocks of a grid with a datacenter. Techs that cost compute can be researched only here. Needs 100 kW.",
};

pub const FIBRE_NODE_RECIPE: Recipe = Recipe {
    output: b(FIBRE_NODE),
    group: Group::Production,
    count: 2,
    inputs: &[(PROCESSOR, 2), (b(GLASS), 4), (PLASTIC, 2), (COPPER_WIRE, 6)],
    blurb: "A post of the data grid. Nodes within 12 blocks of each other join into one grid by themselves; a machine \
            that makes or uses compute (TF) hangs on the nearest node within 5 blocks. Thin light lines show what \
            is joined. A grid shares its compute between its consumers: a shortage slows them all alike.",
};

pub const OPTIMIZER_RECIPE: Recipe = Recipe {
    output: b(OPTIMIZER),
    group: Group::Production,
    count: 1,
    inputs: &[(STEEL_PLATE, 8), (AI_ACCELERATOR, 4), (PROCESSOR, 10), (CIRCUIT, 10), (b(GLASS), 4)],
    blurb:
        "A booster for the data grid, 2×2×2. It always uses 20 TF (a fibre node within 5 blocks, a datacenter on the \
            grid) and 150 kW, and in return every machine within 16 blocks works 25% faster, miners included. Several \
            optimizers do not add up: a machine takes the best one in range. A shortage of power or compute weakens it.",
};
