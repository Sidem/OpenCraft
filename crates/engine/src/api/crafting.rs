//! Hand crafting for the build menu: the recipe table (`recipes.rs`) and crafting from the inventory.

use wasm_bindgen::prelude::*;

use crate::block::AIR;
use crate::recipes::RECIPES;
use crate::{Game, LOCAL};

#[wasm_bindgen]
impl Game {
    pub fn recipe_count(&self) -> u32 {
        RECIPES.len() as u32
    }

    pub fn recipe_output(&self, r: u32) -> u8 {
        RECIPES.get(r as usize).map_or(AIR, |x| x.output)
    }

    pub fn recipe_output_count(&self, r: u32) -> u32 {
        RECIPES.get(r as usize).map_or(0, |x| x.count)
    }

    /// Inputs as flat (item, count) pairs.
    pub fn recipe_inputs(&self, r: u32) -> Vec<u32> {
        RECIPES.get(r as usize).map_or_else(Vec::new, |x| x.inputs.iter().flat_map(|&(i, n)| [i as u32, n]).collect())
    }

    pub fn recipe_blurb(&self, r: u32) -> String {
        RECIPES.get(r as usize).map_or_else(String::new, |x| x.blurb.to_string())
    }

    pub fn can_craft(&self, r: u32) -> bool {
        RECIPES
            .get(r as usize)
            .is_some_and(|x| x.inputs.iter().all(|&(i, n)| self.sim.players[LOCAL].inventory.count(i) >= n))
    }

    /// Crafts recipe `r` up to `times` times from inventory items. Returns how many times it ran.
    pub fn craft(&mut self, r: u32, times: u32) -> u32 {
        let Some(recipe) = RECIPES.get(r as usize) else { return 0 };
        let mut done = 0;
        while done < times && self.can_craft(r) {
            for &(item, n) in recipe.inputs {
                self.sim.players[LOCAL].inventory.remove(item, n);
            }
            let left = self.sim.players[LOCAL].inventory.add(recipe.output, recipe.count);
            if left > 0 {
                self.throw(recipe.output, left);
            }
            done += 1;
        }
        if done > 0 {
            self.pickups.push_back((recipe.output, recipe.count * done));
        }
        done
    }
}
