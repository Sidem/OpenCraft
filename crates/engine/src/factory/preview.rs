//! Machines that are only planned (presentation): a scratch `Factory` holds the ghosts' machines so they draw with the
//! machines' own models (`Machine::model`), seen through the host's blue see-through pass (`ghost_models.rs`).
//!
//! `place_preview` is `Factory::place` without a world: no drill target, no pumpjack well, no pole wall, no wiring
//! (a pole ghost stands on the ground without cables). Nothing here is state a game keeps: the scratch factory is never
//! ticked or saved. To add a machine kind: add its arm here beside `place`'s (the test `every_machine_block_has_a_model`
//! fails otherwise).

use super::*;

impl Factory {
    /// Adds the machine `block` is at `pos` facing `facing`, tiered `tier`, as `place` would, touching no world.
    /// False for a block that is no machine (a plain block has no model: `ghost_models.rs` draws it as a cube).
    pub(crate) fn place_preview(&mut self, block: BlockId, pos: IVec3, facing: u8, tier: u8) -> bool {
        let Some(def) = machine(block) else { return false };
        self.remove(pos);
        let tier = match block {
            LONG_POLE => pole::LONG_TIER,
            COAL_MINER => miner_burner::COAL_TIER,
            _ => tier,
        };
        let at = &mut self.at;
        match def.kind {
            Kind::Belt => self.add_shaped_belt(pos, facing, Shape::of(block), tier.max((block == FAST_BELT) as u8)),
            Kind::Miner => self.add_miner(pos, FACE_BOTTOM as u8, None, tier.max((block == MINER_MK2) as u8)),
            Kind::Storage => self.add_storage(pos),
            Kind::Process => {
                if let Some(spec) = process::spec(block) {
                    let p = Processor { dir: facing % 4, ..Processor::new(pos, spec, tier) };
                    add_to(&mut self.processors, p, at, Slot::Process);
                }
            }
            Kind::Router => {
                add_to(&mut self.routers, Router::new(pos, facing, RouterKind::of(block)), at, Slot::Router)
            }
            Kind::Generator => add_to(&mut self.generators, Generator::new(pos), at, Slot::Generator),
            Kind::Pole => {
                let tier = match block {
                    CABLE => pole::CABLE_TIER,
                    LONG_POLE => pole::LONG_TIER,
                    _ => 0,
                };
                add_to(&mut self.poles, Pole { pos, tier, wall: 0 }, at, Slot::Pole)
            }
            Kind::Lab => add_to(&mut self.labs, Lab::new(pos), at, Slot::Lab),
            Kind::Pipe => add_to(&mut self.pipework, Pipework::new(pos, block, facing), at, Slot::Pipe),
            Kind::Quarry => add_to(&mut self.quarries, Quarry::new(pos, facing), at, Slot::Quarry),
            Kind::Sensor => add_to(&mut self.sensors, Sensor::new(pos, facing), at, Slot::Sensor),
            Kind::Rail => add_to(&mut self.rails, Rail::new(pos, facing), at, Slot::Rail),
            Kind::Node => add_to(&mut self.nodes, Node { pos }, at, Slot::Node),
        }
        if tier > 0 {
            self.set_tier(pos, tier);
        }
        self.dirty = true;
        true
    }

    /// Settles what the models read from neighbours (belt bends, ramps) after a batch of `place_preview` calls.
    pub(crate) fn settle_preview(&mut self) {
        if self.dirty {
            self.relink();
        }
    }
}

#[cfg(test)]
mod tests;
