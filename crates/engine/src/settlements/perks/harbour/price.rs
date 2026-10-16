//! The harbour's price list (step 13.4b): what one of a product is made of in raw materials, and what the harbour asks
//! for it, `PRICE` times that. Worked out from the recipe tables once, on first use (a pure function of the tables, so
//! every peer gets the same list):
//!
//! - an item follows its cheapest route, the one `recipes::recycling` costs (`raw_cost`), the first recipe that matches;
//!   a batch's inputs are shared over what it makes (waste aside);
//! - anything no recipe makes, and anything salvaged from waste alone (sand from slag), is a raw material;
//! - the harbour offers a product made from at least two raw materials, none of them a far ore (`FAR`: bauxite and
//!   uranium, so barter never makes far ores unnecessary), and never the coin or a tool (priced per use, it would come
//!   out worn).
//!
//! To change what is offered: `FAR`, `KINDS_MIN` or `PRICE`.

use std::sync::OnceLock;

use crate::block::{BAUXITE_ORE, URANIUM_ORE};
use crate::item::{ItemId, COIN, ITEM_COUNT};
use crate::recipes::recycling::{is_waste, raw_cost, MILLI};
use crate::recipes::{MACHINE_RECIPES, RECIPES};
use crate::tools::TOOLS;

/// What the harbour asks for a product, in times its raw materials.
pub const PRICE: u64 = 3;
/// Raw materials the harbour never takes, nor products made from them.
const FAR: [ItemId; 2] = [ItemId::block(BAUXITE_ORE), ItemId::block(URANIUM_ORE)];
/// Fewest different raw materials in a product the harbour offers (a plate or glass is not worth a contract).
const KINDS_MIN: usize = 2;
/// Deepest chain followed (a cycle of recipes stops here and is not offered).
const DEPTH: u32 = 40;

/// Each raw material and how many of it, in item order.
pub type Price = Vec<(ItemId, u32)>;

/// What the harbour asks for one `item`, or `None` when it doesn't offer it.
pub fn price(item: ItemId) -> Option<&'static Price> {
    static PRICES: OnceLock<Vec<Option<Price>>> = OnceLock::new();
    PRICES.get_or_init(build).get(item.0 as usize)?.as_ref()
}

/// Millis of each raw material in one of an item, in item order.
type Content = Vec<(ItemId, u64)>;

fn build() -> Vec<Option<Price>> {
    let mut memo = vec![None; ITEM_COUNT];
    (0..ITEM_COUNT)
        .map(|i| {
            let item = ItemId(i as u16);
            let content = content(item, &mut memo, DEPTH)?;
            let tool = TOOLS.iter().any(|t| t.item == item);
            let offered =
                !tool && content.len() >= KINDS_MIN && !content.iter().any(|(r, _)| FAR.contains(r) || *r == COIN);
            offered.then(|| content.iter().map(|&(r, m)| (r, (PRICE * m).div_ceil(u64::from(MILLI)) as u32)).collect())
        })
        .collect()
}

/// The raw materials in one `item`, `None` when nothing makes it or its chain loops.
fn content(item: ItemId, memo: &mut Vec<Option<Option<Content>>>, depth: u32) -> Option<Content> {
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
        for (r, m) in content(input, memo, depth - 1)? {
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
