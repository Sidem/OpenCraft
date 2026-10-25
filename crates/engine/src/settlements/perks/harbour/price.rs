//! The harbour's price list (step 13.4b): what one of a product is made of in raw materials (`recipes::content`), and
//! what the harbour asks for it, `PRICE` times that. Worked out once, on first use (a pure function of the tables, so
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
use crate::recipes::content::content;
use crate::recipes::recycling::MILLI;
use crate::tools::TOOLS;

/// What the harbour asks for a product, in times its raw materials.
pub const PRICE: u64 = 3;
/// Raw materials the harbour never takes, nor products made from them.
const FAR: [ItemId; 2] = [ItemId::block(BAUXITE_ORE), ItemId::block(URANIUM_ORE)];
/// Fewest different raw materials in a product the harbour offers (a plate or glass is not worth a contract).
const KINDS_MIN: usize = 2;

/// Each raw material and how many of it, in item order.
pub type Price = Vec<(ItemId, u32)>;

/// What the harbour asks for one `item`, or `None` when it doesn't offer it.
pub fn price(item: ItemId) -> Option<&'static Price> {
    static PRICES: OnceLock<Vec<Option<Price>>> = OnceLock::new();
    PRICES.get_or_init(build).get(item.0 as usize)?.as_ref()
}

fn build() -> Vec<Option<Price>> {
    (0..ITEM_COUNT)
        .map(|i| {
            let item = ItemId(i as u16);
            let content = content(item)?;
            let tool = TOOLS.iter().any(|t| t.item == item);
            let offered =
                !tool && content.len() >= KINDS_MIN && !content.iter().any(|(r, _)| FAR.contains(r) || *r == COIN);
            offered.then(|| content.iter().map(|&(r, m)| (r, (PRICE * m).div_ceil(u64::from(MILLI)) as u32)).collect())
        })
        .collect()
}
