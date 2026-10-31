//! The house set's hand recipes (`block/house.rs`), all in the build menu's Building section from the start: wood, sand,
//! dirt, glass and stone bricks for the common pieces, iron for the iron door, and a little of the metal that colours each
//! roof (iron red, coal slate, copper green, as in the decor concretes). Never raw ore (a lint in `recipes/tests.rs`).

use crate::block::*;
use crate::item::{COPPER_INGOT, IRON_INGOT, IRON_PLATE, IRON_ROD, STICK};

use super::{b, Group, Recipe};

#[cfg(test)]
mod tests;

pub const WOOD_DOOR_RECIPE: Recipe = Recipe {
    output: b(WOOD_DOOR),
    group: Group::Building,
    count: 2,
    inputs: &[(b(PLANKS), 6), (STICK, 2)],
    blurb: "Right-click to open or close it; an open door can be walked through. Stack two for a doorway.",
};

pub const IRON_DOOR_RECIPE: Recipe = Recipe {
    output: b(IRON_DOOR),
    group: Group::Building,
    count: 2,
    inputs: &[(IRON_PLATE, 4), (IRON_ROD, 2)],
    blurb: "A door that is slower to break than a wooden one. Right-click to open or close it.",
};

pub const WINDOW_RECIPE: Recipe = Recipe {
    output: b(WINDOW),
    group: Group::Building,
    count: 2,
    inputs: &[(b(GLASS), 2), (b(PLANKS), 1)],
    blurb: "A wooden frame round four panes: you see through it, and it keeps the rain out.",
};

pub const PLASTER_RECIPE: Recipe = Recipe {
    output: b(PLASTER),
    group: Group::Building,
    count: 4,
    inputs: &[(b(SAND), 2), (b(DIRT), 2)],
    blurb: "Sand and dirt packed smooth: a pale wall that is easy to make.",
};

pub const TIMBER_FRAME_RECIPE: Recipe = Recipe {
    output: b(TIMBER_FRAME),
    group: Group::Building,
    count: 4,
    inputs: &[(b(PLASTER), 2), (b(PLANKS), 2)],
    blurb: "A plastered wall between dark oak beams, for the upper storeys of a house.",
};

pub const WOOD_FLOOR_RECIPE: Recipe = Recipe {
    output: b(WOOD_FLOOR),
    group: Group::Building,
    count: 4,
    inputs: &[(b(PLANKS), 4)],
    blurb: "Boards laid lengthwise, for floors and walls.",
};

pub const THATCH_RECIPE: Recipe = Recipe {
    output: b(THATCH),
    group: Group::Building,
    count: 4,
    inputs: &[(b(LEAVES), 4), (STICK, 1)],
    blurb: "A straw roof: the cheapest way to keep the weather off.",
};

pub const RED_ROOF_RECIPE: Recipe = Recipe {
    output: b(RED_ROOF),
    group: Group::Building,
    count: 4,
    inputs: &[(b(STONE_BRICKS), 2), (IRON_INGOT, 1)],
    blurb: "Clay-red roof tiles: stone bricks tinted with a little iron.",
};

pub const SLATE_ROOF_RECIPE: Recipe = Recipe {
    output: b(SLATE_ROOF),
    group: Group::Building,
    count: 4,
    inputs: &[(b(STONE_BRICKS), 2), (b(COAL_ORE), 1)],
    blurb: "Dark slate roof tiles: stone bricks tinted with a pinch of coal.",
};

pub const GREEN_ROOF_RECIPE: Recipe = Recipe {
    output: b(GREEN_ROOF),
    group: Group::Building,
    count: 4,
    inputs: &[(b(STONE_BRICKS), 2), (COPPER_INGOT, 1)],
    blurb: "Weathered green roof tiles: stone bricks tinted with a little copper.",
};

pub const CHIMNEY_BRICKS_RECIPE: Recipe = Recipe {
    output: b(CHIMNEY_BRICKS),
    group: Group::Building,
    count: 4,
    inputs: &[(b(STONE_BRICKS), 4), (b(COAL_ORE), 1)],
    blurb: "Sooty red bricks for a chimney stack.",
};

/// The note sign (`block/sign.rs`, `signs/`): not a house piece, but built from the same wood.
pub const SIGN_RECIPE: Recipe = Recipe {
    output: b(SIGN),
    group: Group::Building,
    count: 1,
    inputs: &[(b(PLANKS), 4), (STICK, 2)],
    blurb: "A board two blocks wide for notes. Right-click it to write up to four lines and pick its colours.",
};
