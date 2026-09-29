//! Hand crafting as a timed queue (core state, one per player). A click on a recipe queues an *order*:
//! the recipe, and, when the inventory lacks a part the recipe needs but holds what a hand recipe of
//! that part needs, the crafts of the missing parts first (`plan`, recursively). The order's raw
//! materials leave the inventory at once and wait in the order's own `held` pool; each craft takes its
//! inputs from the pool when it starts, works for `Recipe::hand_ticks`, and puts its output in the pool
//! (a part for a later craft) or in the inventory (`deliver`: what the player asked for). Orders run one
//! after another, one craft at a time.
//!
//! Invariants: an order's steps are in the order they can run, so a craft's inputs are always in the
//! pool when it starts; cancelling (or the player leaving) returns the pool and the inputs of a craft in
//! progress to the inventory, nothing is lost; what doesn't fit is thrown (`SimEvent::Thrown`). Nothing
//! here reads the clock: a craft's progress is counted in ticks, so the state hashes and saves.
//!
//! To change how long crafts take: the constants in `recipes/mod.rs` (`hand_ticks`). To add a recipe a
//! plan can use for a part: a hand recipe with that output; the plan finds it by output item.

use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::{Inventory, Stack};
use crate::item::ItemId;
use crate::recipes::{Recipe, RECIPES};
use crate::research::{Research, Unlock};
use crate::sim::{PlayerId, Sim, SimEvent};

/// Most crafts one order may ask for, and most orders a player may queue.
pub const MAX_TIMES: u32 = 999;
pub const MAX_ORDERS: usize = 12;
/// How many levels of parts a plan may nest (a machine, its parts, their materials).
const MAX_DEPTH: u32 = 6;
/// Bounds on what a save may hold.
const MAX_STEPS: usize = 64;

/// `times` crafts of one recipe. `deliver` is true for the recipe the player chose (its output goes to
/// the inventory), false for a part made for a later step (its output waits in the pool).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Step {
    pub recipe: u16,
    pub times: u32,
    pub deliver: bool,
}

/// What it takes to make something: the crafts in running order and the raw materials they use.
#[derive(Debug)]
pub struct Plan {
    pub steps: Vec<Step>,
    pub leaves: Vec<(ItemId, u32)>,
}

impl Plan {
    /// The crafts of parts (everything but the chosen recipe).
    pub fn part_crafts(&self) -> u32 {
        self.steps.iter().filter(|s| !s.deliver).map(|s| s.times).sum()
    }
}

/// One thing the player asked for, with its parts.
#[derive(Default)]
pub struct Order {
    pub steps: Vec<Step>,
    /// Materials and finished parts waiting for their crafts.
    pub held: Vec<Stack>,
    /// Whether the first step's craft has started (its inputs are out of the pool), and its ticks so far.
    pub busy: bool,
    pub ticks: u32,
    /// Ticks the whole order takes, for its progress bar.
    pub total: u32,
}

impl Order {
    /// The item the player asked for.
    pub fn output(&self) -> ItemId {
        self.steps.last().map_or(ItemId::NONE, |s| RECIPES[s.recipe as usize].output)
    }

    /// How many of it are still to come.
    pub fn amount(&self) -> u32 {
        self.steps.last().map_or(0, |s| s.times * RECIPES[s.recipe as usize].count)
    }

    /// Ticks left, counting the craft in progress.
    fn ticks_left(&self) -> u32 {
        let all: u32 = self.steps.iter().map(|s| s.times * RECIPES[s.recipe as usize].hand_ticks()).sum();
        all.saturating_sub(if self.busy { self.ticks } else { 0 })
    }

    /// Progress of the whole order in thousandths.
    pub fn permille(&self) -> u32 {
        (self.total.saturating_sub(self.ticks_left()) * 1000).checked_div(self.total).unwrap_or(0)
    }
}

/// A player's queued orders; the first one is being worked on.
#[derive(Default)]
pub struct CraftQueue {
    pub orders: Vec<Order>,
}

/// The crafts and materials for `times` of recipe `recipe` from what `inv` holds, or `None` when a
/// material is missing (or the recipe is locked, or a part has no hand recipe).
pub fn plan(inv: &Inventory, research: &Research, recipe: u16, times: u32) -> Option<Plan> {
    let r = RECIPES.get(recipe as usize)?;
    if times == 0 || !research.has(Unlock::Recipe(r.output)) {
        return None;
    }
    let mut p = Planner { inv, research, used: Vec::new(), steps: Vec::new() };
    p.craft(recipe, r, times, 0, true)?;
    Some(Plan { steps: p.steps, leaves: p.used })
}

/// The most crafts of `recipe` (up to `MAX_TIMES`) that `plan` allows.
pub fn max_times(inv: &Inventory, research: &Research, recipe: u16) -> u32 {
    if plan(inv, research, recipe, 1).is_none() {
        return 0;
    }
    let (mut lo, mut hi) = (1, MAX_TIMES);
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if plan(inv, research, recipe, mid).is_some() {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo
}

struct Planner<'a> {
    inv: &'a Inventory,
    research: &'a Research,
    /// What the plan takes from the inventory so far.
    used: Vec<(ItemId, u32)>,
    steps: Vec<Step>,
}

impl Planner<'_> {
    /// Makes sure `n` of `item` will be there: from the inventory first, else crafted by hand.
    fn need(&mut self, item: ItemId, n: u32, depth: u32) -> Option<()> {
        let taken = self.used.iter_mut().find(|u| u.0 == item);
        let have = self.inv.count(item).saturating_sub(taken.as_ref().map_or(0, |u| u.1));
        let take = have.min(n);
        if take > 0 {
            match taken {
                Some(u) => u.1 += take,
                None => self.used.push((item, take)),
            }
        }
        if take == n {
            return Some(());
        }
        let (index, r) = RECIPES
            .iter()
            .enumerate()
            .find(|(_, r)| r.output == item && self.research.has(Unlock::Recipe(r.output)))?;
        self.craft(index as u16, r, (n - take).div_ceil(r.count), depth + 1, false)
    }

    /// Adds the crafts of `times` of `r`, after those of its inputs.
    fn craft(&mut self, index: u16, r: &Recipe, times: u32, depth: u32, deliver: bool) -> Option<()> {
        if depth > MAX_DEPTH {
            return None;
        }
        for &(item, n) in r.inputs {
            self.need(item, n.checked_mul(times)?, depth)?;
        }
        self.steps.push(Step { recipe: index, times, deliver });
        Some(())
    }
}

fn add_to_pool(pool: &mut Vec<Stack>, item: ItemId, count: u32) {
    match pool.iter_mut().find(|s| s.item == item) {
        Some(s) => s.count += count,
        None => pool.push(Stack { item, count }),
    }
}

/// Takes `inputs` out of the pool, or none of them.
fn take_from_pool(pool: &mut Vec<Stack>, inputs: &[(ItemId, u32)]) -> bool {
    let has = |pool: &[Stack], item, n| pool.iter().find(|s| s.item == item).is_some_and(|s| s.count >= n);
    if !inputs.iter().all(|&(item, n)| has(pool, item, n)) {
        return false;
    }
    for &(item, n) in inputs {
        if let Some(s) = pool.iter_mut().find(|s| s.item == item) {
            s.count -= n;
        }
    }
    pool.retain(|s| s.count > 0);
    true
}

/// Puts `count` of `item` in the inventory, throwing what doesn't fit.
fn give(inv: &mut Inventory, player: PlayerId, item: ItemId, count: u32, events: &mut Vec<SimEvent>) {
    let left = inv.add(item, count);
    if left > 0 {
        events.push(SimEvent::Thrown { player, item, count: left });
    }
}

impl CraftQueue {
    /// Queues `times` of `recipe` (fewer if the inventory can't pay for all); false if nothing was queued.
    pub fn enqueue(&mut self, inv: &mut Inventory, research: &Research, recipe: u16, times: u32) -> bool {
        let times = times.min(max_times(inv, research, recipe));
        let Some(plan) = plan(inv, research, recipe, times).filter(|_| self.orders.len() < MAX_ORDERS) else {
            return false;
        };
        let mut held = Vec::new();
        for &(item, n) in &plan.leaves {
            inv.remove(item, n);
            add_to_pool(&mut held, item, n);
        }
        let total = plan.steps.iter().map(|s| s.times * RECIPES[s.recipe as usize].hand_ticks()).sum();
        self.orders.push(Order { steps: plan.steps, held, total, ..Order::default() });
        true
    }

    /// Cancels order `index`, giving back what it holds.
    pub fn cancel(&mut self, index: usize, inv: &mut Inventory, player: PlayerId, events: &mut Vec<SimEvent>) {
        if index >= self.orders.len() {
            return;
        }
        let order = self.orders.remove(index);
        if order.busy {
            for &(item, n) in RECIPES[order.steps[0].recipe as usize].inputs {
                give(inv, player, item, n, events);
            }
        }
        for s in order.held {
            give(inv, player, s.item, s.count, events);
        }
    }

    /// Cancels everything.
    pub fn cancel_all(&mut self, inv: &mut Inventory, player: PlayerId, events: &mut Vec<SimEvent>) {
        while !self.orders.is_empty() {
            self.cancel(0, inv, player, events);
        }
    }

    /// Works one tick on the first order.
    pub fn tick(&mut self, inv: &mut Inventory, player: PlayerId, events: &mut Vec<SimEvent>) {
        let Some(order) = self.orders.first_mut() else { return };
        let step = order.steps[0];
        let r = &RECIPES[step.recipe as usize];
        if !order.busy {
            if !take_from_pool(&mut order.held, r.inputs) {
                return self.cancel(0, inv, player, events);
            }
            (order.busy, order.ticks) = (true, 0);
        }
        order.ticks += 1;
        if order.ticks < r.hand_ticks() {
            return;
        }
        order.busy = false;
        if step.deliver {
            give(inv, player, r.output, r.count, events);
            events.push(SimEvent::Crafted { player, item: r.output, count: r.count });
        } else {
            add_to_pool(&mut order.held, r.output, r.count);
        }
        order.steps[0].times -= 1;
        if order.steps[0].times == 0 {
            order.steps.remove(0);
        }
        if order.steps.is_empty() {
            let done = self.orders.remove(0);
            for s in done.held {
                give(inv, player, s.item, s.count, events);
            }
        }
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.orders.len());
        for o in &self.orders {
            w.count(o.steps.len());
            for s in &o.steps {
                w.u16(s.recipe);
                w.u32(s.times);
                w.bool(s.deliver);
            }
            w.count(o.held.len());
            for s in &o.held {
                s.write_state(w);
            }
            w.bool(o.busy);
            w.u32(o.ticks);
            w.u32(o.total);
        }
    }

    pub fn read_state(r: &mut ByteReader) -> Option<CraftQueue> {
        let n = r.count()?;
        if n > MAX_ORDERS {
            return None;
        }
        let mut orders = Vec::new();
        for _ in 0..n {
            let mut o = Order::default();
            let steps = r.count()?;
            if steps == 0 || steps > MAX_STEPS {
                return None;
            }
            for _ in 0..steps {
                let step = Step { recipe: r.u16()?, times: r.u32()?, deliver: r.bool()? };
                if step.recipe as usize >= RECIPES.len() || step.times == 0 {
                    return None;
                }
                o.steps.push(step);
            }
            for _ in 0..r.count()? {
                o.held.push(Stack::read_state(r)?);
            }
            (o.busy, o.ticks, o.total) = (r.bool()?, r.u32()?, r.u32()?);
            orders.push(o);
        }
        Some(CraftQueue { orders })
    }
}

impl Sim {
    /// One tick of every player's first order.
    pub(crate) fn run_crafting(&mut self) {
        for (i, core) in self.players.iter_mut().enumerate() {
            if let Some(core) = core.as_mut().filter(|c| !c.crafts.orders.is_empty()) {
                core.crafts.tick(&mut core.inventory, PlayerId(i as u8), &mut self.events);
            }
        }
    }
}

#[cfg(test)]
mod tests;
