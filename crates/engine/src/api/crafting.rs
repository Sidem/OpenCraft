//! Hand crafting for the build menu: the recipe table (`recipes/`), which recipes research still
//! locks, queueing crafts and reading the queue (`crafting.rs` has the rules).

use wasm_bindgen::prelude::*;

use crate::action::Action;
use crate::crafting::{craft_ticks, max_times, plan, MAX_ORDERS};
use crate::perks::{player_bonus, Stat};
use crate::recipes::{menu_order, GROUPS, RECIPES};
use crate::research::Unlock;
use crate::settlements::gear_cost;
use crate::{take_apart, Game};

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

    /// Inputs as flat (item, count) pairs: half for a workshop's gear while one is supplied.
    pub fn recipe_inputs(&self, r: u32) -> Vec<u32> {
        let half = self.sim.settlements.gear_works();
        RECIPES
            .get(r as usize)
            .map_or_else(Vec::new, |x| gear_cost(x, half).flat_map(|(i, n)| [i.0 as u32, n]).collect())
    }

    pub fn recipe_blurb(&self, r: u32) -> String {
        RECIPES.get(r as usize).map_or_else(String::new, |x| x.blurb.to_string())
    }

    /// The build-menu section of recipe `r`, an index into the groups (`recipe_group_name`).
    pub fn recipe_group(&self, r: u32) -> u32 {
        RECIPES.get(r as usize).map_or(0, |x| GROUPS.iter().position(|&g| g == x.group).unwrap_or(0) as u32)
    }

    /// Where recipe `r` sorts inside its section: every tier of a building stands together, Mk1 first.
    pub fn recipe_order(&self, r: u32) -> u32 {
        RECIPES.get(r as usize).map_or(0, |_| menu_order(r as usize) as u32)
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
        max_times(self.inventory(), &self.sim.factory.research, self.sim.settlements.gear_works(), r as u16)
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

    /// Seconds one craft of recipe `r` takes by hand, in tenths, at the speed research and worn gear give.
    pub fn recipe_tenths(&self, r: u32) -> u32 {
        RECIPES.get(r as usize).map_or(0, |x| craft_ticks(x, self.craft_speed()) * 10 / crate::TICK_RATE)
    }

    /// How many crafts of parts (not of `r` itself) crafting `r` once would queue first.
    pub fn recipe_part_crafts(&self, r: u32) -> u32 {
        plan(self.inventory(), &self.sim.factory.research, self.sim.settlements.gear_works(), r as u16, 1)
            .map_or(0, |p| p.part_crafts())
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
        let speed = self.craft_speed();
        orders.iter().flat_map(|o| [o.output().0 as u32, o.amount(), o.permille(speed)]).collect()
    }

    /// The steps still to run of the local player's `order`th order, in running order, as flat (item, items
    /// still to come, progress of the craft in progress in thousandths) triples: the parts first, what was
    /// asked for last. Only the first step has progress.
    pub fn craft_steps(&self, order: u32) -> Vec<u32> {
        let Some(o) = self.local_crafts().and_then(|q| q.orders.get(order as usize)) else { return Vec::new() };
        let first = o.step_permille(self.craft_speed());
        let step = |(i, s): (usize, &crate::crafting::Step)| {
            let r = &RECIPES[s.recipe as usize];
            [r.output.0 as u32, s.times * r.count, if i == 0 { first } else { 0 }]
        };
        o.steps.iter().enumerate().flat_map(step).collect()
    }

    /// Cancels the local player's `order`th queued craft.
    pub fn cancel_craft(&mut self, order: u32) {
        self.act(Action::CancelCraft { order: order.min(u16::MAX as u32) as u16 });
    }

    /// What one batch (`recipe_output_count` of the item) of recipe `r` gives back when taken apart, as flat (item,
    /// count) pairs, scrap metal last; empty when it cannot be taken apart (`take_apart.rs`).
    pub fn take_apart_parts(&self, r: u32) -> Vec<u32> {
        let Some(p) = take_apart::parts(r as usize) else { return Vec::new() };
        let scrap = (p.scrap > 0).then_some((crate::item::SCRAP_METAL, p.scrap));
        p.items.into_iter().chain(scrap).flat_map(|(i, n)| [i.0 as u32, n]).collect()
    }

    /// How many batches of recipe `r` the local player could take apart now.
    pub fn take_apart_times(&self, r: u32) -> u32 {
        take_apart::batches(self.inventory(), r as usize)
    }

    /// Takes up to `times` batches of recipe `r` apart at the next tick; returns how many.
    pub fn take_apart(&mut self, r: u32, times: u32) -> u32 {
        let n = times.min(self.take_apart_times(r)).min(u16::MAX as u32);
        if n > 0 {
            self.act(Action::TakeApart { recipe: r as u16, times: n as u16 });
        }
        n
    }
}

impl Game {
    fn local_crafts(&self) -> Option<&crate::crafting::CraftQueue> {
        Some(&self.sim.player(self.local)?.crafts)
    }

    fn craft_speed(&self) -> u32 {
        player_bonus(&self.sim.factory.research, self.inventory(), Stat::Crafting)
    }
}
