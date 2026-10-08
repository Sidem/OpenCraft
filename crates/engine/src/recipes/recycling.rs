//! What a recycler pays for an item ([`millicoins`], thousandths of a coin), worked out from the recipe tables at
//! compile time: no per-item list to keep in step with them. Every item is worth a **whole number of coins**, and the
//! dearest item in the game is worth exactly [`MAX_COINS`] (64).
//!
//! How an item's coins are worked out (all in const evaluation, integers only):
//!
//! 1. Two passes over the recipes give each item a *material cost* `raw` (what the cheapest route consumes, in raw
//!    items) and a *processed value* `deep` (the same with every step of processing doubling what goes in, [`STEP`]).
//!    A batch is shared out over the items it makes; anything no recipe makes (ore, logs, dirt, leaves, saplings) is raw
//!    and costs 1; an item with several recipes takes its cheapest; [`WASTE`] (slag, tailings) is 1 whatever it took, and
//!    a recipe whose inputs are all waste only salvages (what it makes is 1).
//! 2. The curve: `coins = ½ × CURVE_TOP × ((raw × deep) / (raw × deep of the dearest item)) ^ ¼`, with `CURVE_TOP` 128 rounded
//!    to a whole coin first and then halved and rounded down (so the dearest pays `MAX_COINS` 64), at least 1. The product mixes material (how much went in) with processing (how deep the chain is),
//!    and the fourth root flattens the exponential growth of deep chains: a drone port Mk5 pays about 55, not 57,000.
//! 3. A tool is priced per use (its recipe makes `uses` of them); step 2 works on the whole tool (`raw`, `deep` times its
//!    uses), and the whole tool's coins are then spread over its uses, so a worn tool pays in proportion.
//!
//! The coin is worth nothing; the recycler takes everything else. Because the curve is anchored on the dearest item, a
//! new dearer item shifts every value: `tests.rs` prints nothing but checks the anchor, so re-read the table (README
//! "Recycler" section) after adding late-game content.
//!
//! Invariants: the tables are built by const evaluation (relaxation: values only fall, every cycle of recipes grows a
//! value, so it settles); `tests.rs` checks every item has a value, the coin has none and the dearest pays `MAX_COINS`.
//!
//! To change the economy: [`MAX_COINS`] (the scale), [`STEP`] (how much depth counts), the exponent in `coins_of`, or
//! [`WASTE`] for a new byproduct that should never pay more than 1.
use crate::block::{SLAG, TAILINGS};
use crate::item::{ItemId, COIN, ITEM_COUNT};

use crate::tools::TOOLS;

use super::{Recipe, MACHINE_RECIPES, RECIPES};

#[cfg(test)]
mod tests;

/// What each step of processing multiplies the value of its inputs by (the `deep` pass).
pub const STEP: u64 = 2;

/// What the dearest item pays, in coins.
pub const MAX_COINS: u64 = 64;

/// The curve is worked out on this scale (twice `MAX_COINS`), then halved and rounded down.
const CURVE_TOP: u64 = MAX_COINS * 2;

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

static VALUES: [u32; ITEM_COUNT] = finish(&build(1), &build(STEP));

/// Uses of `item` if it is a tool (a recipe makes that many at once), else 1.
const fn uses(item: usize) -> u64 {
    let mut k = 0;
    while k < TOOLS.len() {
        if TOOLS[k].item.0 as usize == item {
            return TOOLS[k].tier.uses as u64;
        }
        k += 1;
    }
    1
}

/// Floor of the square root.
const fn isqrt(n: u128) -> u128 {
    let (mut lo, mut hi) = (0u128, 1u128 << 40);
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if mid * mid <= n {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    lo
}

/// The curve: whole coins for a whole item of material cost `raw` and processed value `deep` (millicoins), given the
/// dearest item's `raw × deep`.
const fn coins_of(raw: u64, deep: u64, top: u128) -> u64 {
    let ratio = (raw as u128 * deep as u128) << 48;
    let root = isqrt(isqrt(ratio / top)) as u64; // (raw × deep / top) ^ ¼, scaled by 2^12
    let coins = ((CURVE_TOP * root + (1 << 11)) >> 12) / 2;
    if coins == 0 {
        1
    } else {
        coins
    }
}

/// Turns the two passes into what one `item` pays, in millicoins.
const fn finish(raw: &[u32; ITEM_COUNT], deep: &[u32; ITEM_COUNT]) -> [u32; ITEM_COUNT] {
    let mut top = 1u128;
    let mut i = 0;
    while i < ITEM_COUNT {
        if i != COIN.0 as usize {
            let n = uses(i);
            let x = (raw[i] as u64 * n) as u128 * (deep[i] as u64 * n) as u128;
            if x > top {
                top = x;
            }
        }
        i += 1;
    }
    let mut out = [0u32; ITEM_COUNT];
    i = 0;
    while i < ITEM_COUNT {
        if i != COIN.0 as usize && raw[i] > 0 {
            let n = uses(i);
            let coins = coins_of(raw[i] as u64 * n, deep[i] as u64 * n, top);
            // Spread over its uses, never less than a millicoin a use.
            let each = (coins * MILLI as u64 + n / 2) / n;
            out[i] = if each == 0 { 1 } else { each as u32 };
        }
        i += 1;
    }
    out
}

const fn build(step: u64) -> [u32; ITEM_COUNT] {
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
            changed |= relax(&mut v, step, r.inputs, r.outputs);
            i += 1;
        }
        i = 0;
        while i < RECIPES.len() {
            let r: &Recipe = &RECIPES[i];
            changed |= relax(&mut v, step, r.inputs, &[(r.output, r.count)]);
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
const fn relax(v: &mut [u64; ITEM_COUNT], step: u64, inputs: &[(ItemId, u32)], outputs: &[(ItemId, u32)]) -> bool {
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
    let each = if all_waste { MILLI as u64 } else { (step * total).div_ceil(shares) };
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
