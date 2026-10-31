//! Which family an item belongs to: the analytics screen filters and colours materials by it, and the backpack sort
//! orders ingredients by it (`inventory/kinds.rs`).
//! Derived from the recipe tables, so a new item sorts itself: an item no recipe makes is a raw material; one a
//! machine makes goes by the first machine recipe (`MACHINE_RECIPES`) it is the main product of; a research pack is science;
//! an item only hand recipes make is a component. The order of [`GROUPS`] is what the screen shows.
//!
//! To add a group: a [`GROUPS`] row (name and colour) and a rule in [`group_of`].

use super::{Category, MACHINE_RECIPES, RECIPES};
use crate::item::ItemId;
use crate::research::PACKS;

/// One group: its name and the colour its chips and list marks use (`0xRRGGBB`).
pub struct GroupInfo {
    pub name: &'static str,
    pub colour: u32,
}

pub const RAW: u8 = 0;
const SMELTED: u8 = 1;
const ORE_PROCESSING: u8 = 2;
const PARTS: u8 = 3;
const COMPONENTS: u8 = 4;
const CHEMICALS: u8 = 5;
const ELECTRONICS: u8 = 6;
const SCIENCE: u8 = 7;
const SPACE: u8 = 8;

pub const GROUPS: [GroupInfo; 9] = [
    GroupInfo { name: "Raw materials", colour: 0xb89b6a },
    GroupInfo { name: "Ingots and bricks", colour: 0xf29e4c },
    GroupInfo { name: "Ore processing", colour: 0x8fbf6a },
    GroupInfo { name: "Basic parts", colour: 0x6ea8fe },
    GroupInfo { name: "Components and kits", colour: 0xc792ea },
    GroupInfo { name: "Chemicals and fuel", colour: 0x4dd0c0 },
    GroupInfo { name: "Electronics", colour: 0xf06bb0 },
    GroupInfo { name: "Science packs", colour: 0xffd166 },
    GroupInfo { name: "Space", colour: 0xe5e7eb },
];

/// The group (an index into [`GROUPS`]) of `item`.
pub fn group_of(item: ItemId) -> u8 {
    if PACKS.contains(&item) {
        return SCIENCE;
    }
    // The recipe it is the main product of; an item that is only ever a byproduct (slag) is bulk, not a product.
    let main = MACHINE_RECIPES.iter().find(|r| r.outputs[0].0 == item);
    let made_by = main.or_else(|| MACHINE_RECIPES.iter().find(|r| r.outputs.iter().any(|&(out, _)| out == item)));
    let made_by = made_by.map(|r| r.category);
    if main.is_none() && made_by.is_some() && item.0 < 256 {
        return RAW;
    }
    match made_by {
        Some(Category::Smelting | Category::Blasting | Category::Electrolysis) => SMELTED,
        // Slag, sand and tailings are plain blocks the crusher and washer give back.
        Some(Category::Crushing | Category::Washing) if item.0 < 256 => RAW,
        Some(Category::Crushing | Category::Washing) => ORE_PROCESSING,
        Some(Category::Pressing) => PARTS,
        Some(Category::Assembly) => COMPONENTS,
        Some(Category::Arc | Category::Fabrication) => ELECTRONICS,
        Some(Category::Launch) => SPACE,
        Some(_) => CHEMICALS,
        None if RECIPES.iter().any(|r| r.output == item) => COMPONENTS,
        None => RAW,
    }
}
