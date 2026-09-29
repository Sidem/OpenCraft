//! Hand recipes of the tools (`tools.rs`): the count of each is its uses, so a craft makes one fresh tool.
//! To add one: a `pub const` here, and its name in `RECIPES`.

use crate::block::*;
use crate::item::*;
use crate::tools::{IRON_TIER, STEEL_TIER, STONE_TIER};

use super::{b, Group, Recipe};

pub const STONE_PICKAXE_RECIPE: Recipe = Recipe {
    output: STONE_PICKAXE,
    group: Group::Tools,
    count: STONE_TIER.uses,
    inputs: &[(b(STONE), 3), (STICK, 2)],
    blurb: "Hold it to break stone and ore twice as fast. Wears out after 150 blocks.",
};

pub const STONE_AXE_RECIPE: Recipe = Recipe {
    output: STONE_AXE,
    group: Group::Tools,
    count: STONE_TIER.uses,
    inputs: &[(b(STONE), 3), (STICK, 2)],
    blurb: "Hold it to chop wood twice as fast. Wears out after 150 blocks.",
};

pub const STONE_SHOVEL_RECIPE: Recipe = Recipe {
    output: STONE_SHOVEL,
    group: Group::Tools,
    count: STONE_TIER.uses,
    inputs: &[(b(STONE), 1), (STICK, 2)],
    blurb: "Hold it to dig dirt, grass and sand twice as fast. Wears out after 150 blocks.",
};

pub const IRON_PICKAXE_RECIPE: Recipe = Recipe {
    output: IRON_PICKAXE,
    group: Group::Tools,
    count: IRON_TIER.uses,
    inputs: &[(IRON_PLATE, 3), (IRON_ROD, 2)],
    blurb: "Breaks stone and ore four times as fast and keeps 4 ore per block instead of 3. Lasts 600 blocks.",
};

pub const IRON_AXE_RECIPE: Recipe = Recipe {
    output: IRON_AXE,
    group: Group::Tools,
    count: IRON_TIER.uses,
    inputs: &[(IRON_PLATE, 3), (IRON_ROD, 2)],
    blurb: "Chops wood four times as fast. Lasts 600 blocks.",
};

pub const IRON_SHOVEL_RECIPE: Recipe = Recipe {
    output: IRON_SHOVEL,
    group: Group::Tools,
    count: IRON_TIER.uses,
    inputs: &[(IRON_PLATE, 1), (IRON_ROD, 2)],
    blurb: "Digs dirt, grass and sand four times as fast. Lasts 600 blocks.",
};

pub const STEEL_PICKAXE_RECIPE: Recipe = Recipe {
    output: STEEL_PICKAXE,
    group: Group::Tools,
    count: STEEL_TIER.uses,
    inputs: &[(STEEL_PLATE, 3), (IRON_ROD, 2)],
    blurb: "Breaks stone and ore six times as fast and keeps 5 ore per block instead of 3. Lasts 1,500 blocks.",
};

pub const STEEL_AXE_RECIPE: Recipe = Recipe {
    output: STEEL_AXE,
    group: Group::Tools,
    count: STEEL_TIER.uses,
    inputs: &[(STEEL_PLATE, 3), (IRON_ROD, 2)],
    blurb: "Chops wood six times as fast. Lasts 1,500 blocks.",
};

pub const STEEL_SHOVEL_RECIPE: Recipe = Recipe {
    output: STEEL_SHOVEL,
    group: Group::Tools,
    count: STEEL_TIER.uses,
    inputs: &[(STEEL_PLATE, 1), (IRON_ROD, 2)],
    blurb: "Digs dirt, grass and sand six times as fast. Lasts 1,500 blocks.",
};
