//! What one of an item is made of in raw materials ([`content`]): millis of each, following the item's cheapest route,
//! the one `recycling::raw_cost` costs (the first recipe that matches), with a batch's inputs shared over what it makes
//! (waste aside). Anything no recipe makes, and anything salvaged from waste alone (sand from slag), is its own raw
//! material. Worked out once on first use, a pure function of the tables, so every peer gets the same answer.
//! Used by the harbour's prices (`settlements/perks/harbour/price.rs`) and by taking items apart (`take_apart.rs`).

use std::sync::OnceLock;

use crate::item::{ItemId, COIN, ITEM_COUNT};

use super::recycling::{is_waste, raw_cost, MILLI};
use super::{MACHINE_RECIPES, RECIPES};

/// Millis of each raw material in one of an item, in item order.
pub type Content = Vec<(ItemId, u64)>;

/// Deepest chain followed (a cycle of recipes stops here and has no content).
const DEPTH: u32 = 40;

/// The raw materials in one `item`, `None` when nothing makes it, it is the coin or its chain loops.
pub fn content(item: ItemId) -> Option<&'static Content> {
    static TABLE: OnceLock<Vec<Option<Content>>> = OnceLock::new();
    TABLE
        .get_or_init(|| {
            let mut memo = vec![None; ITEM_COUNT];
            (0..ITEM_COUNT).map(|i| work(ItemId(i as u16), &mut memo, DEPTH)).collect()
        })
        .get(item.0 as usize)?
        .as_ref()
}

fn work(item: ItemId, memo: &mut Vec<Option<Option<Content>>>, depth: u32) -> Option<Content> {
    if let Some(known) = &memo[item.0 as usize] {
        return known.clone();
    }
    let worked = work_out(item, memo, depth);
    memo[item.0 as usize] = Some(worked.clone());
    worked
}

fn work_out(item: ItemId, memo: &mut Vec<Option<Option<Content>>>, depth: u32) -> Option<Content> {
    let cost = u64::from(raw_cost(item));
    if cost == 0 || item == COIN || depth == 0 {
        return None;
    }
    let raw = vec![(item, u64::from(MILLI))];
    let Some((inputs, shares)) = route(item, cost) else { return Some(raw) };
    if inputs.iter().all(|&(i, _)| is_waste(i)) || is_waste(item) {
        return Some(raw);
    }
    let mut sum: Content = Vec::new();
    for &(input, n) in inputs {
        for (r, m) in work(input, memo, depth - 1)? {
            match sum.iter_mut().find(|e| e.0 == r) {
                Some(e) => e.1 += m * u64::from(n),
                None => sum.push((r, m * u64::from(n))),
            }
        }
    }
    sum.iter_mut().for_each(|e| e.1 = e.1.div_ceil(shares));
    crate::math::sort_small_by_key(&mut sum, |e| e.0 .0);
    Some(sum)
}

/// The first recipe that makes `item` at its cheapest `cost`: its inputs and how many items share them. `None` for a
/// raw material (no recipe makes it).
fn route(item: ItemId, cost: u64) -> Option<(&'static [(ItemId, u32)], u64)> {
    let matches = |inputs, outputs: &[(ItemId, u32)]| {
        let cost_each = outputs.iter().any(|o| o.0 == item).then(|| batch_cost(inputs, outputs)).flatten();
        cost_each.filter(|c| c.0 == cost).map(|c| (inputs, c.1))
    };
    let machine = MACHINE_RECIPES.iter().find_map(|r| matches(r.inputs, r.outputs));
    machine.or_else(|| RECIPES.iter().find_map(|r| matches(r.inputs, &[(r.output, r.count)])))
}

/// What each item a batch makes costs (as `recipes::recycling` works it out with no processing step) and how many
/// share it; `None` when an input has no cost or nothing but waste comes out.
fn batch_cost(inputs: &[(ItemId, u32)], outputs: &[(ItemId, u32)]) -> Option<(u64, u64)> {
    let mut total = 0;
    for &(i, n) in inputs {
        let c = u64::from(raw_cost(i));
        if c == 0 && i != COIN {
            return None;
        }
        total += c * u64::from(n);
    }
    let shares: u64 = outputs.iter().filter(|o| !is_waste(o.0)).map(|o| u64::from(o.1)).sum();
    if shares == 0 {
        return None;
    }
    let all_waste = inputs.iter().all(|&(i, _)| is_waste(i));
    Some((if all_waste { u64::from(MILLI) } else { total.div_ceil(shares) }, shares))
}
