//! The river town's perk (step 13.4): drone ports within `RELAY_RANGE` blocks of a supplied river town's depot reach
//! and fly `factory::RELAY` times as far (`Hangar::relayed`, derived each tick).
//! To change the area: `RELAY_RANGE`.

use crate::sim::Sim;

use super::super::RIVER_TOWN;

/// How near a drone port's anchor must be to a river town's depot, in blocks, horizontally.
pub const RELAY_RANGE: i32 = 128;

impl Sim {
    /// Marks the drone ports a supplied river town relays this tick.
    pub(crate) fn relay_ports(&mut self) {
        let towns: Vec<_> =
            self.settlements.towns.iter().filter(|t| t.kind == RIVER_TOWN && t.perk_on()).map(|t| t.pos).collect();
        self.factory.relay_ports(|p| {
            towns.iter().any(|t| {
                let (dx, dz) = (i64::from(p.x - t.x), i64::from(p.z - t.z));
                dx * dx + dz * dz <= i64::from(RELAY_RANGE * RELAY_RANGE)
            })
        });
    }
}
