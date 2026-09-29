//! Hand crafting for the build menu: the recipe table (`recipes/`), which recipes research still
//! locks, queueing crafts and reading the queue (`crafting.rs` has the rules).

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::crafting::{max_times, plan, MAX_ORDERS};
use crate::recipes::{GROUPS, RECIPES};
use crate::research::Unlock;
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

    /// The build-menu section of recipe `r`, an index into the groups (`recipe_group_name`).
    pub fn recipe_group(&self, r: u32) -> u32 {
        RECIPES.get(r as usize).map_or(0, |x| GROUPS.iter().position(|&g| g == x.group).unwrap_or(0) as u32)
    }

    pub fn recipe_group_count(&self) -> u32 {
        GROUPS.len() as u32
    }

    pub fn recipe_group_name(&self, g: u32) -> String {
        GROUPS.get(g as usize).map_or("", |g| g.name()).to_string()
    }

    /// How many times the local player can craft recipe `r` right now, counting the parts it can make
    /// first (0 while research locks it).
    pub fn craftable_times(&self, r: u32) -> u32 {
        max_times(self.inventory(), &self.sim.factory.research, r as u16)
    }

    /// The tech (`tech_*`) that must be researched before recipe `r` can be crafted, or -1.
    pub fn recipe_locked_by(&self, r: u32) -> i32 {
        let locked =
            RECIPES.get(r as usize).and_then(|x| self.sim.factory.research.locked_by(Unlock::Recipe(x.output)));
        locked.map_or(-1, i32::from)
    }

    pub fn can_craft(&self, r: u32) -> bool {
        self.craftable_times(r) > 0
    }

    /// Seconds one craft of recipe `r` takes by hand, in tenths.
    pub fn recipe_tenths(&self, r: u32) -> u32 {
        RECIPES.get(r as usize).map_or(0, |x| x.hand_ticks() * 10 / crate::TICK_RATE)
    }

    /// How many crafts of parts (not of `r` itself) crafting `r` once would queue first.
    pub fn recipe_part_crafts(&self, r: u32) -> u32 {
        plan(self.inventory(), &self.sim.factory.research, r as u16, 1).map_or(0, |p| p.part_crafts())
    }

    /// Queues up to `times` crafts of recipe `r` at the next tick, with the crafts of missing parts.
    /// Returns how many will be queued.
    pub fn craft(&mut self, r: u32, times: u32) -> u32 {
        let full = self.local_crafts().is_some_and(|q| q.orders.len() >= MAX_ORDERS);
        let n = if full { 0 } else { times.min(self.craftable_times(r)) };
        if n > 0 {
            self.act(Action::Craft { recipe: r as u16, times: n });
        }
        n
    }

    /// The local player's queue as flat (item, amount still to come, progress in thousandths) triples,
    /// the order being worked on first.
    pub fn craft_queue(&self) -> Vec<u32> {
        let orders = self.local_crafts().map_or(&[][..], |q| &q.orders[..]);
        orders.iter().flat_map(|o| [o.output().0 as u32, o.amount(), o.permille()]).collect()
    }

    /// Cancels the local player's `order`th queued craft.
    pub fn cancel_craft(&mut self, order: u32) {
        self.act(Action::CancelCraft { order: order.min(u16::MAX as u32) as u16 });
    }
}

impl Game {
    fn local_crafts(&self) -> Option<&crate::crafting::CraftQueue> {
        Some(&self.sim.player(self.local)?.crafts)
    }
}
