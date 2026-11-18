//! The LED lamp's and the street lamp's hand recipes (`block/led.rs`, `led/`, `street/`), unlocked by the LED Lamps tech
//! (`research/later.rs`). To add a recipe: a `pub const` here and its name in `RECIPES`.

use crate::block::{GLASS, LED, STREET};
use crate::item::{ItemId, COPPER_WIRE, IRON_PLATE, IRON_ROD, SILICON};

use super::{Group, Recipe};

pub const LED_RECIPE: Recipe = Recipe {
    output: ItemId::block(LED),
    group: Group::Building,
    count: 2,
    inputs: &[(ItemId::block(GLASS), 1), (SILICON, 1), (COPPER_WIRE, 2)],
    blurb: "A lamp that lights about 24 blocks full: right-click it to pick a colour and one of three brightnesses. \
            Needs no power.",
};

pub const STREET_RECIPE: Recipe = Recipe {
    output: ItemId::block(STREET),
    group: Group::Building,
    count: 1,
    inputs: &[(IRON_ROD, 4), (IRON_PLATE, 2), (ItemId::block(LED), 1)],
    blurb: "A post four blocks tall with an arm over what you were facing. Its light is an LED's, from high up where \
            walls and machines don't shade it. Needs no power.",
};
