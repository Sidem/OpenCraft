//! Hand crafting for the build menu: the recipe table (`recipes.rs`), which recipes research still
//! locks, and crafting from the inventory.

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

    /// The tech (`tech_*`) that must be researched before recipe `r` can be crafted, or -1.
    pub fn recipe_locked_by(&self, r: u32) -> i32 {
        let locked = RECIPES.get(r as usize).and_then(|x| self.sim.factory.research.locked_by(x.output));
        locked.map_or(-1, i32::from)
    }

    pub fn can_craft(&self, r: u32) -> bool {
        self.affordable(r) > 0
    }

    /// Crafts recipe `r` up to `times` times from inventory items, at the next tick. Returns how many
    /// times the inventory can pay for it now, i.e. how many will run.
    pub fn craft(&mut self, r: u32, times: u32) -> u32 {
        let n = times.min(self.affordable(r));
        if n > 0 {
            self.act(Action::Craft { recipe: r as u16, times: n });
        }
        n
    }
}

impl Game {
    /// How many times the local player can craft recipe `r` now (0 while research locks it).
    fn affordable(&self, r: u32) -> u32 {
        let Some(x) = RECIPES.get(r as usize) else { return 0 };
        if self.sim.factory.research.locked_by(x.output).is_some() {
            return 0;
        }
        x.affordable(self.inventory())
    }
}
