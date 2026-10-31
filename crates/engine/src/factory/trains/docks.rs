//! A stopped train trading with a dock (`process/docks.rs`): a loading dock's items go into the wagons, the wagons'
//! cargo goes into an unloading dock. A dock is a rail node (`rail/pieces.rs`), so a train stops at one it reaches,
//! with its first wagon over the dock where the track goes on far enough (`motion.rs`).
//! Up to `TRADE_PER_TICK` items a tick; the train drives on once nothing has been traded for `DWELL_TICKS` (it is
//! full or empty, or the dock is, or the dock is gone).

use crate::math::IVec3;

use super::super::buffer::Buffer;
use super::super::process::{Pick, Processor, Status};
use super::super::Factory;
use super::{Train, DWELL_TICKS};

/// Items moved between a dock and a train each tick.
const TRADE_PER_TICK: u32 = 8;

impl Factory {
    /// The dock a train stopping at node `at` trades with (an index into `processors`): the dock that node is.
    pub(super) fn dock_at(&self, at: IVec3) -> Option<usize> {
        self.dock_piece(at)
    }

    /// Stops the stream of the dock `t` is stopped on, as `t` is taken off the track.
    pub(super) fn leave_dock(&mut self, t: &Train) {
        let at = self.stop_node(t).filter(|_| t.idle.is_some());
        if let Some(i) = at.and_then(|n| self.dock_at(n)) {
            self.processors[i].status = Status::NoInput;
        }
    }

    /// One tick of a stopped train: trades with its dock, and drives on once it has idled long enough (to the next
    /// dock of its schedule, if it has one).
    pub(super) fn trade(&mut self, t: &mut Train) {
        let Some(idle) = t.idle else { return };
        let dock = self.stop_node(t).and_then(|n| self.stop_dock(t, n));
        let moved = dock.map_or(0, |i| swap(&mut self.processors[i], &mut t.cargo));
        if let Some(i) = dock {
            // Working while goods move (its lamp and the stream of goods in its model).
            self.processors[i].status = if moved > 0 { Status::Working } else { Status::NoInput };
        }
        let idle = if moved > 0 { 0 } else { idle + 1 };
        t.idle = (idle < DWELL_TICKS).then_some(idle);
        if t.idle.is_none() && !t.schedule.is_empty() {
            t.next = ((usize::from(t.next) + 1) % t.schedule.len()) as u8;
        }
    }
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
