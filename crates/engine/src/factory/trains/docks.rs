//! A stopped train trading with a dock (`process/docks.rs`): a loading dock's items go into the wagons, the wagons'
//! cargo goes into an unloading dock. Up to `TRADE_PER_TICK` items a tick; the train drives on once nothing has been
//! traded for `DWELL_TICKS` (it is full or empty, or the dock is, or the dock is gone).

use crate::math::IVec3;

use super::super::buffer::Buffer;
use super::super::process::{Pick, Processor};
use super::super::Factory;
use super::{Train, DWELL_TICKS};

/// How close a dock's nearest cell must be to a rail node for trains to stop there, in blocks on each axis.
const STOP_REACH: i32 = 2;
/// Items moved between a dock and a train each tick.
const TRADE_PER_TICK: u32 = 8;

impl Factory {
    /// The dock a train stopping at node `at` trades with (an index into `processors`).
    pub(super) fn dock_at(&self, at: IVec3) -> Option<usize> {
        self.processors.iter().position(|p| p.spec.pick.docks() && near(p, at))
    }

    /// One tick of a stopped train: trades with its dock, and drives on once it has idled long enough (to the next
    /// dock of its schedule, if it has one).
    pub(super) fn trade(&mut self, t: &mut Train) {
        let Some(idle) = t.idle else { return };
        let node = t.path.last().map(|e| e.1);
        let dock = node.and_then(|n| self.stop_dock(t, n));
        let moved = dock.map_or(0, |i| swap(&mut self.processors[i], &mut t.cargo));
        let idle = if moved > 0 { 0 } else { idle + 1 };
        t.idle = (idle < DWELL_TICKS).then_some(idle);
        if t.idle.is_none() && !t.schedule.is_empty() {
            t.next = ((usize::from(t.next) + 1) % t.schedule.len()) as u8;
        }
    }
}

/// Whether a cell of `dock` is within `STOP_REACH` blocks, on each axis, of the node `at`.
pub(super) fn near(dock: &Processor, at: IVec3) -> bool {
    let close = |c: IVec3| {
        (c.x - at.x).abs() <= STOP_REACH && (c.y - at.y).abs() <= STOP_REACH && (c.z - at.z).abs() <= STOP_REACH
    };
    dock.cells().into_iter().any(close)
}

/// Moves up to `TRADE_PER_TICK` items between the dock's store and the cargo (into the cargo for a loading dock);
/// returns how many.
fn swap(dock: &mut Processor, cargo: &mut Buffer) -> u32 {
    let (from, to) = if dock.spec.pick == Pick::Load { (&mut dock.out, cargo) } else { (cargo, &mut dock.out) };
    let mut left = TRADE_PER_TICK;
    for slot in 0..from.slots.len() {
        let s = from.slots[slot];
        let n = if s.is_empty() { 0 } else { s.count.min(left).min(to.space_for(s.item)) };
        if n > 0 {
            to.add(s.item, n);
            from.take(slot, n);
            left -= n;
        }
        if left == 0 {
            break;
        }
    }
    TRADE_PER_TICK - left
}
