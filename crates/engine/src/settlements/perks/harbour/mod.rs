//! The harbour's perk (step 13.4b): contract work. The player picks, in its depot's panel, a product their machines
//! have made (`Made`) that the harbour offers (`price.rs`); the harbour then takes `PRICE` times the product's raw
//! materials, at its upkeep's rate (`super::rate` raw items a minute, `BATCH` a batch), and sends a product out of the
//! front each time the price is paid.
//!
//! Invariants: what is paid toward the next product (`Town::paid`, a count per price line) is saved, since a dear
//! product takes many batches; choosing another product starts over (the goods in the perk slots stay, to take by
//! hand). Its depot wants each raw material up to what is still owed (a stack at most), so the perk slots never fill
//! with goods it no longer needs.

mod made;
mod price;

pub use made::Made;
pub use price::{price, PRICE};

use crate::factory::Buffer;
use crate::item::{stack_size, ItemId};
use crate::math::IVec3;
use crate::sim::SimEvent;

use super::BATCH;
use crate::settlements::{Settlements, Town, HARBOUR};

impl Settlements {
    /// Whether a harbour may take on `item`: one it offers that the players' machines have made.
    pub fn offers(&self, item: ItemId) -> bool {
        self.made.has(item) && price(item).is_some()
    }

    /// Sets what the harbour whose depot is at `pos` sends out (`NONE` for nothing), starting its payment over; does
    /// nothing for another town or a product it doesn't offer.
    pub fn set_product(&mut self, pos: IVec3, item: ItemId) {
        let offered = item == ItemId::NONE || self.offers(item);
        if let Some(t) = self.towns.iter_mut().find(|t| t.pos == pos && t.kind == HARBOUR && !t.dormant) {
            if offered && t.product != item {
                t.product = item;
                t.paid = price(item).map_or_else(Vec::new, |p| vec![0; p.len()]);
            }
        }
    }
}

/// Each raw material still owed toward the next product, up to a stack.
pub(super) fn wants(town: &Town) -> Vec<(ItemId, u32)> {
    let Some(price) = price(town.product) else { return Vec::new() };
    let owed = price.iter().zip(&town.paid).map(|(&(item, n), &paid)| (item, (n - paid.min(n)).min(stack_size(item))));
    owed.filter(|o| o.1 > 0).collect()
}

/// One batch: up to `BATCH` raw materials paid in, line by line, then the product out once all is paid (and it fits).
pub(super) fn batch(town: &mut Town, input: &mut Buffer, out: &mut Buffer, events: &mut Vec<SimEvent>) {
    let Some(price) = price(town.product) else { return };
    let mut left = BATCH;
    for (&(item, n), paid) in price.iter().zip(&mut town.paid) {
        let take = (n - (*paid).min(n)).min(input.count(item)).min(left);
        if take > 0 {
            input.remove(item, take);
            events.push(SimEvent::Consumed { item, count: take });
            *paid += take;
            left -= take;
        }
    }
    if price.iter().zip(&town.paid).all(|(l, &p)| p >= l.1) && out.can_hold(town.product, 1) {
        out.add(town.product, 1);
        events.push(SimEvent::Produced { item: town.product, count: 1 });
        town.paid.fill(0);
    }
}
