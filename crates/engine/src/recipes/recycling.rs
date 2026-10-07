//! What a recycler pays for an item ([`millicoins`], thousandths of a coin so that a plank or one use of a tool can pay
//! a fraction), worked out from the recipe tables at compile time: no per-item list to keep in step with them.
//!
//! The rule: **every step of processing doubles the value of what goes in.** A batch (a machine or a hand recipe)
//! is worth [`STEP`] times its inputs, shared out over the items it makes:
//!
//! `value(item) = STEP × Σ value(input) × count / Σ count of items made` (rounded up to a millicoin)
//!
//! - Anything no recipe makes (ore, logs, dirt, leaves, saplings) is raw and worth 1 coin.
//! - An item with several recipes is worth what its *cheapest* recipe gives, so an ingot is 2 whether it was smelted
//!   from ore, crushed ore or washed ore (what an item is worth does not depend on how it was made, and a better
//!   route cannot print coins). Plates (2 ingots) are 8, a rod (1 ingot) 4, screws (4 from a rod) 2 each, a plank (4 from a log) half a coin.
//! - [`WASTE`] (slag, tailings) is worth 1 coin for good. A recipe whose inputs are all waste only salvages: what it makes
//!   is worth 1 coin, so sand crushed from slag stays 1. Waste a batch leaves over takes no share of the batch's value,
//!   and what is later made *from* that sand (glass) is an ordinary step again.
//! - The coin is worth nothing; the recycler takes everything else.
//!
//! Invariants: the table is built by const evaluation (relaxation: values only fall, every cycle of recipes grows a
//! value, so it settles); `tests.rs` checks every item has a value and that no recipe could lower one.
//!
//! To change the economy: [`STEP`], or [`WASTE`] for a new byproduct that should never pay more than 1.

use crate::block::{SLAG, TAILINGS};
use crate::item::{ItemId, COIN, ITEM_COUNT};

use super::{Recipe, MACHINE_RECIPES, RECIPES};

#[cfg(test)]
mod tests;

/// What each step of processing multiplies the value of its inputs by.
pub const STEP: u64 = 2;

/// Millicoins in a coin.
pub const MILLI: u32 = 1000;

/// Byproducts that are only ever worth 1 coin, and whatever is salvaged from them.
pub const WASTE: [ItemId; 2] = [ItemId::block(SLAG), ItemId::block(TAILINGS)];

/// Thousandths of a coin a recycler pays for one `item` (0 for the coin itself and for ids that are not items).
pub fn millicoins(item: ItemId) -> u32 {
    VALUES.get(item.0 as usize).copied().unwrap_or(0)
}

/// Whole coins one `item` pays (rounded down; see [`millicoins`]).
#[cfg(test)]
pub fn coins(item: ItemId) -> u32 {
    millicoins(item) / MILLI
}

/// Whether `item` is waste (worth 1 whatever it took to make).
pub const fn is_waste(item: ItemId) -> bool {
    let mut i = 0;
    while i < WASTE.len() {
        if WASTE[i].0 == item.0 {
            return true;
        }
        i += 1;
    }
    false
}

/// Not worked out yet.
const UNSET: u64 = u64::MAX;

static VALUES: [u32; ITEM_COUNT] = build();

const fn build() -> [u32; ITEM_COUNT] {
    let mut v = [UNSET; ITEM_COUNT];
    let mut made = [false; ITEM_COUNT];
    let mut i = 0;
    while i < MACHINE_RECIPES.len() {
        mark(&mut made, MACHINE_RECIPES[i].outputs);
        i += 1;
    }
    i = 0;
    while i < RECIPES.len() {
        made[RECIPES[i].output.0 as usize] = true;
        i += 1;
    }
    // Raw items, waste and the coin start settled; everything else waits for a recipe.
    i = 0;
    while i < ITEM_COUNT {
        if !made[i] || is_waste(ItemId(i as u16)) {
            v[i] = MILLI as u64;
        }
        i += 1;
    }
    v[COIN.0 as usize] = 0;
    let mut changed = true;
    while changed {
        changed = false;
        i = 0;
        while i < MACHINE_RECIPES.len() {
            let r = &MACHINE_RECIPES[i];
            changed |= relax(&mut v, r.inputs, r.outputs);
            i += 1;
        }
        i = 0;
        while i < RECIPES.len() {
            let r: &Recipe = &RECIPES[i];
            changed |= relax(&mut v, r.inputs, &[(r.output, r.count)]);
            i += 1;
        }
    }
    let mut out = [0u32; ITEM_COUNT];
    i = 0;
    while i < ITEM_COUNT {
        out[i] = if v[i] == UNSET { 0 } else { v[i] as u32 };
        i += 1;
    }
    out
}

const fn mark(made: &mut [bool; ITEM_COUNT], outputs: &[(ItemId, u32)]) {
    let mut k = 0;
    while k < outputs.len() {
        made[outputs[k].0 .0 as usize] = true;
        k += 1;
    }
}

/// Lowers the value of what one batch makes, if the batch's inputs are all settled; whether anything changed.
const fn relax(v: &mut [u64; ITEM_COUNT], inputs: &[(ItemId, u32)], outputs: &[(ItemId, u32)]) -> bool {
    let (mut total, mut all_waste) = (0u64, true);
    let mut k = 0;
    while k < inputs.len() {
        let value = v[inputs[k].0 .0 as usize];
        if value == UNSET {
            return false;
        }
        total += value * inputs[k].1 as u64;
        all_waste &= is_waste(inputs[k].0);
        k += 1;
    }
    let mut shares = 0u64;
    k = 0;
    while k < outputs.len() {
        if !is_waste(outputs[k].0) {
            shares += outputs[k].1 as u64;
        }
        k += 1;
    }
    if shares == 0 {
        return false;
    }
    let each = if all_waste { MILLI as u64 } else { (STEP * total).div_ceil(shares) };
    let mut changed = false;
    k = 0;
    while k < outputs.len() {
        let at = outputs[k].0 .0 as usize;
        if !is_waste(outputs[k].0) && at != COIN.0 as usize && each < v[at] {
            v[at] = each;
            changed = true;
        }
        k += 1;
    }
    changed
}
