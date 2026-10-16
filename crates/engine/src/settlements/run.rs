//! The towns' tick (step 13.2): each depot takes in only its town's goods, a built town uses up its upkeep each minute
//! (or stalls until it is there), and while it is not stalled the rest goes toward its open order.
//!
//! Invariants: runs inside `Sim::step` only (`sync_depots` before the tick's actions, `step_towns` after the factory),
//! so it stays deterministic. The depot's store is the factory's; a town only takes from it, through `depot_store`.

use crate::factory::Buffer;
use crate::sim::{Sim, SimEvent};
use crate::TICK_RATE;

use super::{Town, ORDERS};

/// Ticks between two upkeeps.
const MINUTE: u64 = 60 * TICK_RATE as u64;

impl Sim {
    /// Tells each depot what its town takes in now.
    pub(crate) fn sync_depots(&mut self) {
        for t in &self.settlements.towns {
            self.factory.set_depot_wants(t.pos, t.wants());
        }
    }

    /// Runs each town on what its depot holds, plans the buildings of a stage just built, and flies its drones.
    pub(crate) fn step_towns(&mut self) {
        for i in 0..self.settlements.towns.len() {
            let t = &mut self.settlements.towns[i];
            if let Some(store) = self.factory.depot_store(t.pos) {
                t.step(store, self.tick, &mut self.events);
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
    pub fn step(&mut self, store: &mut Buffer, tick: u64, events: &mut Vec<SimEvent>) {
        let upkeep = self.upkeep();
        if !upkeep.is_empty() && tick >= self.due {
            self.stalled = upkeep.iter().any(|&(item, n)| store.count(item) < n);
            if !self.stalled {
                for &(item, n) in &upkeep {
                    store.remove(item, n);
                    events.push(SimEvent::Consumed { item, count: n });
                }
                self.due = tick + MINUTE;
            }
        }
        if self.stalled {
            return;
        }
        let order = self.order();
        for (&(item, want), got) in order.iter().zip(&mut self.delivered) {
            let reserve = upkeep.iter().find(|u| u.0 == item).map_or(0, |u| u.1);
            let n = (want - *got).min(store.count(item).saturating_sub(reserve));
            if n > 0 {
                store.remove(item, n);
                events.push(SimEvent::Consumed { item, count: n });
                *got += n;
            }
        }
        if !order.is_empty() && order.iter().zip(&self.delivered).all(|(l, &d)| d >= l.1) {
            self.built += 1;
            self.delivered = vec![0; ORDERS.get(self.built as usize).map_or(0, |o| o.len())];
            if self.built == 1 {
                self.due = tick + MINUTE;
            }
        }
    }
}
