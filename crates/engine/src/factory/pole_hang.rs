//! Where a pole's wires hang (presentation only: nothing here is stored, hashed or read by the core). A wire leaves the
//! top of an insulator, not the pole's middle: an ordinary pole has one at each end of its crossarm (`INSULATOR_X`,
//! `INSULATOR_Y`, the model's own constants in `pole.rs`), a long-distance tower two levels of arm tips on each side
//! (`pole_tower::HANGS`), a cable its middle. The arm cannot turn (a pole stores no facing), so it runs along x.
//!
//! A wire picks its end on a pole by two rules. Side: the insulator on the side the other end lies towards
//! (`Pole::side_towards`; straight along z counts as east, so a north-south line is parallel at both ends). Level: wires
//! on one side take the levels in the order they are listed (`Power::wires`, then the machines' leads), and past the
//! last level they share it, fanned `FAN` apart along the arm towards the post so they do not overlap. The order is the
//! list's, so it never depends on the camera: `Factory::for_each_wire` counts every wire, shown or not.
//! To add a pole model: its levels in `Pole::levels` and its centre in `Pole::hang_centre`, beside the model's constants.

use super::links::Slot;
use super::pole::{Pole, INSULATOR_X, INSULATOR_Y, LONG_FOOTPRINT, POLE_RISE};
use super::{pole_tower, Factory};
use crate::math::{IVec3, Vec3};

/// A wire rests on top of an insulator's cap, not inside it.
const HANG_LIFT: f64 = 0.04;
/// Wires sharing a level are this far apart along the arm, and the fan wraps after `FAN_STEPS` of them.
const FAN: f64 = 0.04;
const FAN_STEPS: usize = 4;

/// An ordinary pole's one level: how far out its insulator stands and how high its cap's top is, from the cell's middle.
const POLE_LEVELS: [(f64, f64); 1] = [(INSULATOR_X, INSULATOR_Y + POLE_RISE + HANG_LIFT)];

impl Pole {
    /// The side (0 west, 1 east) a wire to the point `to` leaves on.
    pub fn side_towards(&self, to: Vec3) -> usize {
        usize::from(to.x >= self.wire_point().x)
    }

    /// Where the `rank`th wire on `side` hangs, in the world. A cable has no arm: its middle.
    pub fn hang_point(&self, side: usize, rank: usize) -> Vec3 {
        if self.is_cable() {
            return self.wire_point();
        }
        let levels = self.levels();
        let (reach, y) = levels[rank % levels.len()];
        let fan = ((rank / levels.len()) % FAN_STEPS) as f64 * FAN;
        let out = if side == 1 { reach - fan } else { fan - reach };
        self.hang_centre() + Vec3::new(out, y, 0.0)
    }

    /// Every level of one side, lowest first: how far out the insulator stands and how high, from `hang_centre`.
    fn levels(&self) -> &'static [(f64, f64)] {
        if self.is_long() {
            &pole_tower::HANGS
        } else {
            &POLE_LEVELS
        }
    }

    /// The point the levels are measured from: the cell's middle, a tower's footprint middle.
    fn hang_centre(&self) -> Vec3 {
        let middle = self.pos.as_vec3() + Vec3::new(0.5, 0.5, 0.5);
        if self.is_long() {
            middle + LONG_FOOTPRINT.centre(0)
        } else {
            middle
        }
    }
}

/// Where the first wire towards `to` would hang from a pole of `tier` and `wall` anchored at `pos`: the preview of a
/// pole not placed yet (`power_tools.rs`).
pub fn ghost_hang(pos: IVec3, tier: u8, wall: u8, to: Vec3) -> Vec3 {
    let pole = Pole { pos, tier, wall };
    pole.hang_point(pole.side_towards(to), 0)
}

impl Factory {
    /// The end of the wire from pole `i` towards `to`, and one more wire counted on its side in `used` (one pair of
    /// counters a pole): `for_each_wire` calls this for every wire, shown or not, in the list's order.
    pub(super) fn hang_end(&self, i: u32, to: Vec3, used: &mut [[u8; 2]]) -> Vec3 {
        let pole = &self.poles[i as usize];
        let side = pole.side_towards(to);
        let rank = used[i as usize][side];
        used[i as usize][side] = rank.saturating_add(1);
        pole.hang_point(side, rank as usize)
    }

    /// Where a new wire from the pole anchored at `pos` towards `to` would hang (the wiring preview): the next free
    /// level on its side, counting the wires to other poles it has now. `None` when no pole is anchored there.
    pub fn pole_hang(&self, pos: IVec3, to: Vec3) -> Option<Vec3> {
        let Some(&Slot::Pole(i)) = self.at.get(&pos) else { return None };
        let pole = &self.poles[i as usize];
        let side = pole.side_towards(to);
        let on_side = |&&(a, b): &&(u32, u32)| {
            let other = if a == i {
                b
            } else if b == i {
                a
            } else {
                return false;
            };
            pole.side_towards(self.poles[other as usize].wire_point()) == side
        };
        Some(pole.hang_point(side, self.power.wires.iter().filter(on_side).count()))
    }
}

#[cfg(test)]
mod tests;
