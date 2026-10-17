//! How a machine was placed, read back: the facing and tier a blueprint copies (`blueprint/`), and whether
//! it is a working machine, and how much working machinery is near a point. Read-only; `Factory::place` is the inverse.

use crate::math::{IVec3, Vec3};

use super::links::Slot;
use super::miner::MinerStatus;
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
            Slot::Miner(_)
            | Slot::Storage(_)
            | Slot::Generator(_)
            | Slot::Pole(_)
            | Slot::Lab(_)
            | Slot::Rail(_)
            | Slot::Node(_) => 0,
        };
        Some((facing % 4, self.tiered_at(pos).map_or(0, |(_, tier)| tier)))
    }

    /// Whether a working machine (a processor, miner, generator or lab, not logistics, storage or poles) is
    /// anchored at `pos`: what the blueprint prompt counts (`blueprint/repeat.rs`).
    pub fn works_at(&self, pos: IVec3) -> bool {
        match self.at.get(&pos) {
            Some(Slot::Process(i)) => self.processors[*i as usize].pos == pos,
            Some(Slot::Miner(_) | Slot::Generator(_) | Slot::Lab(_)) => true,
            _ => false,
        }
    }

    /// How loud the machines at work around `at` are (ambient sound, `surroundings`): each processor with a
    /// batch in progress, running miner or generator with output counts `1 - distance / radius`, summed.
    pub fn din_near(&self, at: Vec3, radius: f64) -> f64 {
        let near = |p: IVec3| {
            let d = (Vec3::new(f64::from(p.x) + 0.5, f64::from(p.y) + 0.5, f64::from(p.z) + 0.5) - at).length();
            (1.0 - d / radius).max(0.0)
        };
        let processors = self.processors.iter().filter(|p| p.batch.is_some()).map(|p| near(p.pos));
        let miners = self.miners.iter().filter(|m| m.status == MinerStatus::Running).map(|m| near(m.pos));
        let generators = self.generators.iter().filter(|g| g.output > 0).map(|g| near(g.pos));
        processors.chain(miners).chain(generators).sum()
    }
}
