//! The item guide for the host (`item_guide.rs`, `ui/item-guide.ts`): every way to get an item in this world, what
//! uses it, and which items have a page at all. Recipes are named by index; the host reads them with the `recipe_*`
//! and `machine_recipe_*` getters.

use wasm_bindgen::prelude::*;

use crate::item::{ItemId, ITEM_COUNT};
use crate::item_guide::{uses, ways, Way};
use crate::Game;

#[wasm_bindgen]
impl Game {
    /// Every way to get `item`, one a line, fields split by tabs: `found<TAB>how`, `hand<TAB>recipe` or
    /// `machine<TAB>machine recipe<TAB>the item that places the machine`.
    pub fn item_ways(&self, item: u16) -> String {
        let lines = ways(self.sim.world.generator(), ItemId(item)).into_iter().map(|w| match w {
            Way::Found(text) => format!("found\t{text}"),
            Way::Hand(r) => format!("hand\t{r}"),
            Way::Machine { recipe, machine } => format!("machine\t{recipe}\t{}", machine.0),
        });
        lines.collect::<Vec<_>>().join("\n")
    }

    /// What uses `item`, as flat (kind, recipe) pairs: kind 0 a hand recipe, 1 a machine recipe.
    pub fn item_uses(&self, item: u16) -> Vec<u32> {
        uses(ItemId(item)).into_iter().flat_map(|(machine, r)| [u32::from(machine), r as u32]).collect()
    }

    /// Every item the guide has something to say about (a way to get it or a use), in id order.
    pub fn guide_items(&self) -> Vec<u16> {
        let gen = self.sim.world.generator();
        let known = |i: &ItemId| i.is_valid() && (!ways(gen, *i).is_empty() || !uses(*i).is_empty());
        (1..ITEM_COUNT as u16).map(ItemId).filter(known).map(|i| i.0).collect()
    }
}
