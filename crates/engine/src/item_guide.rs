//! How to get any item, and what it is for (presentation, queries only): the build menu's guide (`ui/item-guide.ts`).
//! The ways come from the tables, so a new recipe shows up by itself: hand recipes (`recipes::RECIPES`), machine
//! recipes (`MACHINE_RECIPES`, made in the first machine whose spec takes the category: `factory::SPECS` order), ore
//! as this world's generator lays it (`ore_guide::ore_way`), and [`FOUND`] for what the world or a machine of its own
//! gives without a recipe (ground, trees, ruins, the pumpjack, the recycler).
//!
//! Invariant: every item a recipe, a fuel slot or a town asks for, and every block the world drops, has at least one
//! way, so a player who needs something can always find out where it comes from (`tests.rs` names any that has none).
//!
//! To add a source no recipe describes: a row in `FOUND`.

use crate::block::{self, BlockId};
use crate::factory::{first_maker, tiers};
use crate::item::{ItemId, COIN, CRUDE_CANISTER, SCRAP_METAL};
use crate::recipes::{Category, MACHINE_RECIPES, RECIPES};
use crate::worldgen::WorldGen;

/// One way to come by an item.
#[derive(Clone, PartialEq, Debug)]
pub enum Way {
    /// Found or dug in the world, or given by a machine without a recipe: how, in a sentence or two.
    Found(String),
    /// Hand recipe `RECIPES[i]`.
    Hand(usize),
    /// Machine recipe `MACHINE_RECIPES[recipe]`, made in `machine` (the item that places the first machine for it).
    Machine { recipe: usize, machine: ItemId },
}

/// Every way to get `item` in the world `gen` makes: found first (the simple way for raw materials), then hand
/// recipes, then machine recipes in table order.
pub fn ways(gen: &WorldGen, item: ItemId) -> Vec<Way> {
    let mut out = Vec::new();
    if let Some(text) = found(gen, item) {
        out.push(Way::Found(text));
    }
    out.extend(RECIPES.iter().enumerate().filter(|(_, r)| r.output == item).map(|(i, _)| Way::Hand(i)));
    for (i, r) in MACHINE_RECIPES.iter().enumerate() {
        // A launch's outputs are the empty canisters coming back: not a way to make canisters.
        if r.category == Category::Launch || !r.outputs.iter().any(|o| o.0 == item) {
            continue;
        }
        if let Some(machine) = machine_for(r.category) {
            out.push(Way::Machine { recipe: i, machine });
        }
    }
    out
}

/// What uses `item`: the hand recipes (`false`, index) and machine recipes (`true`, index) that take it, in table order.
pub fn uses(item: ItemId) -> Vec<(bool, usize)> {
    let hand = RECIPES.iter().enumerate().filter(|(_, r)| r.inputs.iter().any(|x| x.0 == item));
    let machine = MACHINE_RECIPES.iter().enumerate().filter(|(_, r)| r.inputs.iter().any(|x| x.0 == item));
    hand.map(|(i, _)| (false, i)).chain(machine.map(|(i, _)| (true, i))).collect()
}

/// The item that places the first machine making recipes of `category`.
pub fn machine_for(category: Category) -> Option<ItemId> {
    let block = first_maker(category)?;
    Some(tiers::item_of(block, 0).unwrap_or(ItemId::block(block)))
}

/// Sources that are not recipes, by the item they give.
const FOUND: &[(ItemId, &str)] = &[
    (b(block::STONE), "Dig it almost anywhere below the soil, best with a pickaxe."),
    (b(block::DIRT), "Dig the ground: grass and soil give dirt."),
    (b(block::SAND), "Dig it on beaches, along rivers and in deserts."),
    (b(block::LOG), "Chop the trunk of any tree, best with an axe."),
    (b(block::LEAVES), "Break the leaves of a tree."),
    (b(block::SAPLING), "Leaves now and then leave one behind when broken or when they wither."),
    (b(block::BIRCH_SAPLING), "Birch leaves now and then leave one behind when broken or when they wither."),
    (b(block::PINE_SAPLING), "Pine needles now and then leave one behind when broken or when they wither."),
    (b(block::CACTUS), "Grows in deserts: break it."),
    (b(block::GRANITE), "The rock under the highlands: dig into granite hills and mountains."),
    (b(block::SANDSTONE), "The rock under desert sand: dig below it."),
    (b(block::BASALT), "The black rock of basalt fields."),
    (b(block::SPENT_ROCK), "What ore turns into once a miner has drawn its share out: dig it where a deposit ran dry."),
    (SCRAP_METAL, "Break the rusted plating, girders and machines of ruins. Taking iron things apart gives some too."),
    (CRUDE_CANISTER, "A pumpjack set on oil sand fills empty canisters with crude oil."),
    (COIN, "A recycler pays coins for the items it destroys."),
];

/// How `item` is found or dug, if it is.
fn found(gen: &WorldGen, item: ItemId) -> Option<String> {
    let as_block = (item.0 < 256).then_some(item.0 as BlockId);
    if let Some(b) = as_block.filter(|&b| block::is_ore(b)) {
        let lacks = "This world was made before it was in the game: only a new world has it.";
        return Some(crate::ore_guide::ore_way(gen, b).unwrap_or_else(|| lacks.to_string()));
    }
    FOUND.iter().find(|f| f.0 == item).map(|f| f.1.to_string())
}

const fn b(id: BlockId) -> ItemId {
    ItemId::block(id)
}

#[cfg(test)]
mod tests;
