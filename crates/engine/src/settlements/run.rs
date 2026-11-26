//! The towns' tick (step 13.2): each depot takes in only its town's goods, a built town uses up its upkeep each minute
//! (or stalls until it is there), and while it is not stalled the rest goes toward its open order.
//!
//! Invariants: runs inside `Sim::step` only (`sync_depots` before the tick's actions, `step_towns` after the factory),
//! so it stays deterministic. The depot's store is the factory's; a town only takes from it, through `depot_store`.

use crate::factory::{Buffer, Want};
use crate::sim::{Sim, SimEvent};
use crate::TICK_RATE;

use super::{Line, Made, Town, ORDERS};

/// Ticks between two upkeeps.
pub(crate) const MINUTE: u64 = 60 * TICK_RATE as u64;

impl Sim {
    /// Tells each depot what its town and its perk take in now, and marks the drone ports a river town relays.
    pub(crate) fn sync_depots(&mut self) {
        let made = &self.settlements.made;
        for t in &self.settlements.towns {
            let mut wants: Vec<Want> =
                t.wants(made).into_iter().map(|(item, cap)| Want { item, cap, perk: false }).collect();
            wants.extend(t.perk_wants());
            self.factory.set_depot_wants(t.pos, wants);
        }
        self.relay_ports();
    }

    /// Runs each town on what its depot holds, then its perk, plans the buildings of a stage just built, and flies its
    /// drones.
    pub(crate) fn step_towns(&mut self) {
        for i in 0..self.settlements.towns.len() {
            let (t, made) = (&mut self.settlements.towns[i], &self.settlements.made);
            if t.dormant {
                continue;
            }
            if let Some(store) = self.factory.depot_store(t.pos) {
                t.step(store, made, self.tick, &mut self.events);
            }
            if let Some((input, out)) = self.factory.depot_perk(t.pos) {
                t.step_perk(input, out, self.tick, &mut self.events);
            }
            while self.settlements.towns[i].planned < self.settlements.towns[i].built {
                self.plan_stage(i);
            }
            self.step_builders(i);
        }
    }
}

impl Town {
    /// One tick: the upkeep when due, then deliveries toward the open order unless stalled.
    pub fn step(&mut self, store: &mut Buffer, made: &Made, tick: u64, events: &mut Vec<SimEvent>) {
        let upkeep = self.upkeep(made);
        if !upkeep.is_empty() && tick >= self.due {
            self.stalled = !take_upkeep(store, &upkeep, events);
            if !self.stalled {
                self.due = tick + MINUTE;
            }
        }
        if self.stalled {
            return;
        }
        if deliver(self.order(), &mut self.delivered, &upkeep, store, events) {
            self.built += 1;
            self.delivered = vec![0; ORDERS.get(self.built as usize).map_or(0, |o| o.len())];
            if self.built == 1 {
                self.due = tick + MINUTE;
            }
        }
    }
}

/// Uses up a minute of `upkeep` from `store`, all or nothing: false (and nothing taken) when any of it is short.
pub(crate) fn take_upkeep(store: &mut Buffer, upkeep: &[Line], events: &mut Vec<SimEvent>) -> bool {
    if upkeep.iter().any(|&(item, n)| store.count_like(item) < n) {
        return false;
    }
    for &(item, n) in upkeep {
        for s in store.remove_like(item, n) {
            events.push(SimEvent::Consumed { item: s.item, count: s.count });
        }
    }
    true
}

/// Moves from `store` toward an open `order` what it still misses, keeping back a minute of `upkeep`; `delivered` has
/// one count per line. True once every line is complete (never for an empty order).
pub(crate) fn deliver(
    order: &[Line],
    delivered: &mut [u32],
    upkeep: &[Line],
    store: &mut Buffer,
    events: &mut Vec<SimEvent>,
) -> bool {
    for (&(item, want), got) in order.iter().zip(delivered.iter_mut()) {
        let reserve = upkeep.iter().find(|u| u.0 == item).map_or(0, |u| u.1);
        let n = (want - *got).min(store.count_like(item).saturating_sub(reserve));
        if n > 0 {
            for s in store.remove_like(item, n) {
                events.push(SimEvent::Consumed { item: s.item, count: s.count });
            }
            *got += n;
        }
    }
    !order.is_empty() && order.iter().zip(delivered.iter()).all(|(l, &d)| d >= l.1)
}
