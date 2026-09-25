//! Hand crafting for the build menu: the recipe table (`recipes.rs`) and crafting from the inventory.

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::recipes::RECIPES;
use crate::Game;

#[wasm_bindgen]
impl Game {
    pub fn recipe_count(&self) -> u32 {
        RECIPES.len() as u32
    }

    pub fn recipe_output(&self, r: u32) -> u16 {
        RECIPES.get(r as usize).map_or(0, |x| x.output.0)
    }

    pub fn recipe_output_count(&self, r: u32) -> u32 {
        RECIPES.get(r as usize).map_or(0, |x| x.count)
    }

    /// Inputs as flat (item, count) pairs.
    pub fn recipe_inputs(&self, r: u32) -> Vec<u32> {
        RECIPES.get(r as usize).map_or_else(Vec::new, |x| x.inputs.iter().flat_map(|&(i, n)| [i.0 as u32, n]).collect())
    }

    pub fn recipe_blurb(&self, r: u32) -> String {
        RECIPES.get(r as usize).map_or_else(String::new, |x| x.blurb.to_string())
    }

    pub fn can_craft(&self, r: u32) -> bool {
        RECIPES.get(r as usize).is_some_and(|x| x.affordable(self.inventory()) > 0)
    }

    /// Crafts recipe `r` up to `times` times from inventory items, at the next tick. Returns how many
    /// times the inventory can pay for it now, i.e. how many will run.
    pub fn craft(&mut self, r: u32, times: u32) -> u32 {
        let Some(recipe) = RECIPES.get(r as usize) else { return 0 };
        let n = times.min(recipe.affordable(self.inventory()));
        if n > 0 {
            self.act(Action::Craft { recipe: r as u16, times: n });
        }
        n
    }
}
