//! What kind of thing an item is, for the backpack and box sort (`sort_stacks`): buildings, tools, building blocks,
//! ingredients, raw materials and natural blocks, in the order [`KINDS`] lists them. Derived from the recipe tables,
//! so a new item sorts itself: an item a build-menu recipe makes goes by that recipe's section (and, inside it, by its
//! place in the menu); a tool is a tool; anything else a recipe or machine makes is an ingredient, grouped as the
//! analytics screen groups it; an ore, or a block only machines give back, is a raw material; any other block is
//! natural. `inventory/tests.rs` checks every item lands where it should.
//!
//! To add a kind: a [`KINDS`] row and a rule in `place_of`.

use crate::block::is_ore;
use crate::factory::tiers;
use crate::item::ItemId;
use crate::recipes::item_groups::{group_of, RAW as RAW_GROUP};
use crate::recipes::{menu_order, Group, MACHINE_RECIPES, RECIPES};
use crate::research::PACKS;
use crate::tools;

pub const BUILDINGS: u8 = 0;
pub const TOOLS: u8 = 1;
pub const BUILDING_BLOCKS: u8 = 2;
pub const INGREDIENTS: u8 = 3;
pub const RAW: u8 = 4;
pub const NATURAL: u8 = 5;

/// The kinds' names, indexed by [`kind_of`].
pub const KINDS: [&str; 6] =
    ["Buildings", "Tools", "Building blocks", "Ingredients", "Raw materials", "Natural blocks"];

/// The kind (an index into [`KINDS`]) of `item`.
#[cfg(test)]
pub fn kind_of(item: ItemId) -> u8 {
    place_of(item).0
}

/// Where `item` sorts: its kind, then its section inside the kind (build-menu section or ingredient group), then its
/// place in the build menu, then its id.
#[inline(never)]
pub fn sort_key(item: ItemId) -> u64 {
    let (kind, section, order) = place_of(item);
    (kind as u64) << 48 | (section as u64) << 32 | (order.min(0xffff) as u64) << 16 | item.0 as u64
}

/// (kind, section, menu order) of `item`.
fn place_of(item: ItemId) -> (u8, u8, usize) {
    // A machine's Mk2 or Mk3 sorts with its Mk1.
    let made = tiers::placed_by(item).map_or(item, |(block, _)| ItemId::block(block));
    let hand = (0..RECIPES.len()).filter(|&r| RECIPES[r].output == made).min_by_key(|&r| menu_order(r));
    let hand = hand.map(|r| (RECIPES[r].group, menu_order(r)));
    let ingredient = (INGREDIENTS, group_of(item), 0);
    if tools::tool(item).is_some() || matches!(hand, Some((Group::Tools, _))) {
        return (TOOLS, 0, hand.map_or(0, |h| h.1));
    }
    match hand {
        Some((Group::Materials, _)) => ingredient,
        // A science pack is made from the science section but goes into labs.
        Some(_) if PACKS.contains(&item) => ingredient,
        Some((Group::Building, order)) => (BUILDING_BLOCKS, 0, order),
        Some((group, order)) => (BUILDINGS, group as u8, order),
        None if item.0 >= 256 => ingredient,
        None if is_ore(item.0 as u8) => (RAW, 0, 0),
        None if MACHINE_RECIPES.iter().any(|m| m.outputs.iter().any(|&(out, _)| out == item)) => {
            // Slag, sand and tailings are what machines give back; glass, bricks and concrete are made to build with.
            if group_of(item) == RAW_GROUP {
                (RAW, 1, 0)
            } else {
                (BUILDING_BLOCKS, 1, 0)
            }
        }
        None => (NATURAL, 0, 0),
    }
}
