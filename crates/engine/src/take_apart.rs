//! Taking a hand-crafted item apart (user request 2026-10-25: undo a wrong craft, such as a Miner made where a Coal
//! Miner was meant). Instant, by hand, from the build menu (`Action::TakeApart`). One *batch* is what one craft makes
//! (`Recipe::count`: four belts, one machine) and gives back [`KEEP_PERCENT`] of each of the recipe's inputs, rounded
//! to the nearest whole item. Of the iron in what is kept back, a third returns as scrap metal (one per
//! [`SCRAP_IRON`]), which a smelter melts 3 to an iron ingot; the rest is lost.
//!
//! Invariants (`tests.rs` checks every hand recipe; a linear program over every recipe checked once that no item gets
//! cheaper in any ore: `DEV_PLAN.md` change log, 2026-10-25): a batch gives back no more of any raw material (`recipes::content`) than
//! the cheapest route to that many of the item, so no loop through crafting, machines or the upgrader makes material
//! or turns one raw material into another (scrap comes only from iron, so stone never becomes iron); workshop gear
//! gives back from its halved cost (`settlements::gear_cost`); tools are never taken apart (a workshop mends worn ones
//! whole). An item whose hand recipe uses more of something than its cheapest route cannot be taken apart.
//!
//! To change how much comes back: [`KEEP_PERCENT`] (`tests.rs` needs at least 80 for a wrong Miner to become a Coal
//! Miner) or [`SCRAP_IRON`].

use crate::block::IRON_ORE;
use crate::inventory::Inventory;
use crate::item::{ItemId, IRON_INGOT, SCRAP_METAL};
use crate::recipes::content::{content, Content};
use crate::recipes::{Recipe, RECIPES};
use crate::settlements::gear_cost;
use crate::sim::{PlayerId, SimEvent};
use crate::tools;

#[cfg(test)]
mod tests;

/// Percent of each input a batch gives back (rounded to the nearest item).
pub const KEEP_PERCENT: u32 = 80;
/// Millis of iron ore (in `recipes::content`) kept back for each scrap metal given: a third of what the scrap melts to.
pub const SCRAP_IRON: u64 = 1000;
/// Scrap metal a smelter melts into one iron ingot (`recipes/scrap.rs`).
pub const SCRAP_PER_INGOT: u64 = 3;

/// What one batch of a recipe gives back.
#[derive(Debug, PartialEq, Eq)]
pub struct Parts {
    pub items: Vec<(ItemId, u32)>,
    pub scrap: u32,
}

/// What one batch of hand recipe `recipe` gives back, or `None` when it cannot be taken apart (a tool, a recipe that
/// uses more of a raw material than the item's cheapest route, or no such recipe).
pub fn parts(recipe: usize) -> Option<Parts> {
    let r = RECIPES.get(recipe)?;
    if tools::tool(r.output).is_some() {
        return None;
    }
    let (mut items, mut lost) = (Vec::new(), Content::new());
    for (item, paid) in gear_cost(r, true) {
        let back = (paid * KEEP_PERCENT * 2 + 100) / 200;
        add(&mut lost, content(item)?, (paid - back) as u64, 1);
        if back > 0 {
            items.push((item, back));
        }
    }
    let iron = lost.iter().find(|e| e.0 == ItemId::block(IRON_ORE)).map_or(0, |e| e.1);
    let p = Parts { items, scrap: (iron / SCRAP_IRON) as u32 };
    fits(&p, r).then_some(p)
}

/// How many batches of `recipe` the inventory holds (0 when it cannot be taken apart).
pub fn batches(inv: &Inventory, recipe: usize) -> u32 {
    match parts(recipe) {
        Some(_) => inv.count(RECIPES[recipe].output) / RECIPES[recipe].count,
        None => 0,
    }
}

/// Takes up to `times` batches of `recipe` apart from `inv`: the items leave, the parts and scrap come in (what does not
/// fit is thrown). Does nothing when the recipe cannot be taken apart or none is held.
pub fn take_apart(inv: &mut Inventory, recipe: usize, times: u32, player: PlayerId, events: &mut Vec<SimEvent>) {
    let times = times.min(batches(inv, recipe));
    let Some(p) = parts(recipe).filter(|_| times > 0) else { return };
    let r: &Recipe = &RECIPES[recipe];
    inv.remove(r.output, times * r.count);
    let scrap = (p.scrap > 0).then_some((SCRAP_METAL, p.scrap));
    for (item, n) in p.items.into_iter().chain(scrap) {
        let count = n * times;
        let left = inv.add(item, count);
        if left < count {
            events.push(SimEvent::Gained { player, item, count: count - left });
        }
        if left > 0 {
            events.push(SimEvent::Thrown { player, item, count: left });
        }
    }
}

/// Whether `p` holds no more of any raw material than `r.count` of `r.output` by its cheapest route.
fn fits(p: &Parts, r: &Recipe) -> bool {
    let (Some(whole), Some(ingot)) = (content(r.output), content(IRON_INGOT)) else { return false };
    let mut back = Content::new();
    for &(item, n) in &p.items {
        let Some(c) = content(item) else { return false };
        add(&mut back, c, n as u64, 1);
    }
    add(&mut back, ingot, p.scrap as u64, SCRAP_PER_INGOT);
    back.iter().all(|&(raw, m)| m <= whole.iter().find(|e| e.0 == raw).map_or(0, |e| e.1) * r.count as u64)
}

/// Adds `n / per` of content `c` to `sum` (rounded up per raw material, so the check errs towards refusing).
fn add(sum: &mut Content, c: &Content, n: u64, per: u64) {
    for &(raw, m) in c {
        let m = (m * n).div_ceil(per);
        match sum.iter_mut().find(|e| e.0 == raw) {
            Some(e) => e.1 += m,
            None => sum.push((raw, m)),
        }
    }
}
