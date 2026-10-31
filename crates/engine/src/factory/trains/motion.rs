//! How fast a train goes: it gathers speed by `ACCEL` a tick up to `STEP_MM` and brakes by `BRAKE` a tick for the
//! next place it must stop (`room_ahead`), so it eases into and out of every stop. Braking from full speed takes
//! `BRAKE_ZONE` at most, shorter than any track (`MIN_SPAN`), so only the end of its edge, or the spot past a dock it
//! stops at, can need it. A train with wagons stops `STOP_PAST` beyond a dock (its first wagon over the dock) when
//! the track goes on that far, else at the dock itself (`passes`); `stop_node` says which dock a stopped train is at.
//!
//! Whether a train halts at the end of its edge is decided once, as it comes within `BRAKE_ZONE`, and kept in `halts`
//! until it takes the next edge (a signal that clears meanwhile lets it speed up again; one that turns red stops it
//! short), so the costly look ahead (`way`) runs once an edge.
//!
//! Invariants: integer millimetres only (`isqrt`), so every peer agrees; the speed is never 0 for a moving train,
//! so it always reaches its stop. `odo` counts the distance driven (wrapping), for the wheels.

use crate::math::IVec3;

use super::signals::Way;
use super::{Factory, Train, CAR_MM, LOCO_MM, STEP_MM};

/// Speed gained a tick, and lost a tick braking (mm a tick, each tick): from standing to full speed in 50 ticks.
const ACCEL: i64 = 3;
const BRAKE: i64 = 5;
/// The braking distance from full speed (`STEP_MM² / 2 BRAKE`) and one tick's run: where braking may begin.
const BRAKE_ZONE: i64 = STEP_MM * STEP_MM / (2 * BRAKE) + STEP_MM;
/// How far past a dock a train with wagons stops: its first wagon's middle over the dock.
pub(super) const STOP_PAST: i64 = LOCO_MM + CAR_MM / 2;

impl Factory {
    /// Moves a train that is not stopped one tick, at a speed that eases toward its next stop.
    pub(super) fn drive(&self, t: &mut Train) {
        let Some(&(a, b)) = t.path.last() else { return };
        let Some(len) = self.edge_mm(a, b) else { return };
        let cap = self.room_ahead(t, a, b, len).map_or(STEP_MM, |left| isqrt(2 * BRAKE * left));
        t.speed = (t.speed + ACCEL).min(STEP_MM).min(cap).max(1);
        t.odo = t.odo.wrapping_add(t.speed as u32);
        let mm = t.speed;
        self.advance(t, mm, (a, b), len);
    }

    /// The distance to where `t`, on the edge from `a` to `b` `len` long, must stop, once that is within `BRAKE_ZONE`.
    fn room_ahead(&self, t: &mut Train, a: IVec3, b: IVec3, len: i64) -> Option<i64> {
        if t.cars > 0 && t.head < STOP_PAST && len > STOP_PAST && self.stop_dock(t, a).is_some() {
            return Some(STOP_PAST - t.head);
        }
        let left = len - t.head;
        if left > BRAKE_ZONE {
            return None;
        }
        let halts = match t.halts {
            Some(h) => h,
            None => {
                let h = if t.cars > 0 && self.stop_dock(t, b).is_some() {
                    !self.passes(t, a, b)
                } else {
                    !matches!(self.way(t, a, b), Way::Go(_))
                };
                t.halts = Some(h);
                h
            }
        };
        halts.then_some(left)
    }

    /// Whether a train with wagons reaching the dock at `b` from `a` drives on to stop `STOP_PAST` beyond it.
    pub(super) fn passes(&self, t: &mut Train, a: IVec3, b: IVec3) -> bool {
        match self.way(t, a, b) {
            Way::Go(c) => self.edge_mm(b, c).is_some_and(|l| l > STOP_PAST),
            _ => false,
        }
    }

    /// Whether `t`, on the edge from `a` with its front at `head`, has just reached the spot past the dock at `a`.
    pub(super) fn past_dock(&self, t: &mut Train, a: IVec3, before: i64, len: i64) -> bool {
        t.cars > 0 && before < STOP_PAST && t.head >= STOP_PAST && len > STOP_PAST && self.stop_dock(t, a).is_some()
    }

    /// The node a stopped train is at: the end of its last edge, or its start when it stopped past a dock there.
    pub(super) fn stop_node(&self, t: &Train) -> Option<IVec3> {
        let &(a, b) = t.path.last()?;
        Some(if self.edge_mm(a, b)? == t.head { b } else { a })
    }
}

/// The integer square root (rounded down).
fn isqrt(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut x = (n as f64).sqrt() as i64;
    while x * x > n {
        x -= 1;
    }
    while (x + 1) * (x + 1) <= n {
        x += 1;
    }
    x
}
