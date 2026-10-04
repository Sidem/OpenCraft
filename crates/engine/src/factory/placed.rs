//! How a machine was placed, read back: the facing and tier a blueprint copies (`blueprint/`).
//! Read-only; `Factory::place` is the inverse.

use crate::math::IVec3;

use super::links::Slot;
use super::Factory;

impl Factory {
    /// The facing and tier the machine anchored at `pos` was placed with; `None` unless a machine's anchor
    /// cell (not another cell of a multi-block machine) is at `pos`. Tier is 0 for untiered machines.
    pub fn placed_as(&self, pos: IVec3) -> Option<(u8, u8)> {
        let slot = *self.at.get(&pos)?;
        let facing = match slot {
            Slot::Belt(i) => self.belts[i as usize].dir,
            Slot::Process(i) => {
                let p = &self.processors[i as usize];
                if p.pos != pos {
                    return None;
                }
                p.dir
            }
            Slot::Router(i) => self.routers[i as usize].dir,
            Slot::Sensor(i) => self.sensors[i as usize].dir,
            Slot::Pipe(i) => self.pipework[i as usize].facing,
            Slot::Quarry(i) => self.quarries[i as usize].facing,
            Slot::Miner(_) | Slot::Storage(_) | Slot::Generator(_) | Slot::Pole(_) | Slot::Lab(_) => 0,
        };
        Some((facing % 4, self.tiered_at(pos).map_or(0, |(_, tier)| tier)))
    }
}
