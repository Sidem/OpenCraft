//! The processors of each kind that has a pass of its own each tick (power sources, sensing, bonuses, pods, research,
//! water), as index lists, so those passes visit only their few machines instead of walking every processor.
//!
//! Invariants: derived from the processor list by `Factory::relink` (`Kinds::of`), never saved. A list stays right
//! until the factory is dirty again: placing or removing a processor and changing its tier (which can change its
//! `Energy`) all mark it dirty. Indices ascend, so a pass visits its machines in the order a full walk would (the
//! deterministic core, DEV_PLAN section 3.4). What a pass reads that can change between relinks (a pad's channel,
//! a center's unit) it checks itself.
//!
//! To give a new kind a pass of its own: a list here and its arm in `of`.

use crate::block::{GROUND_STATION, OPTIMIZER, SWARM_HUB};

use super::{Energy, Processor};

#[derive(Default)]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(in crate::factory) struct Kinds {
    /// Power sources and stores, by `Energy` (`Power::balance`).
    pub solar: Vec<u32>,
    pub accumulator: Vec<u32>,
    pub hydro: Vec<u32>,
    pub rectenna: Vec<u32>,
    pub reactor: Vec<u32>,
    pub turbine: Vec<u32>,
    pub diesel: Vec<u32>,
    /// Ground stations (`ground.rs`), optimizers and swarm hubs (`optimizer.rs`, `hub.rs`), by block.
    pub ground: Vec<u32>,
    pub optimizer: Vec<u32>,
    pub hub: Vec<u32>,
    /// Pod launchers and landing pads (`pods.rs`), research centers and AI labs (`center.rs`).
    pub launcher: Vec<u32>,
    pub pad: Vec<u32>,
    pub center: Vec<u32>,
    /// Every processor with a water inlet (`steam::draw_water`).
    pub water: Vec<u32>,
}

impl Kinds {
    /// The lists for `processors`.
    pub fn of(processors: &[Processor]) -> Kinds {
        let mut k = Kinds::default();
        for (i, p) in processors.iter().enumerate() {
            let i = i as u32;
            let by_energy = match p.energy() {
                Energy::Solar => Some(&mut k.solar),
                Energy::Accumulator => Some(&mut k.accumulator),
                Energy::Hydro => Some(&mut k.hydro),
                Energy::Rectenna => Some(&mut k.rectenna),
                Energy::Reactor => Some(&mut k.reactor),
                Energy::Turbine => Some(&mut k.turbine),
                Energy::Diesel => Some(&mut k.diesel),
                _ => None,
            };
            if let Some(list) = by_energy {
                list.push(i);
            }
            let by_block = match p.spec.block {
                GROUND_STATION => Some(&mut k.ground),
                OPTIMIZER => Some(&mut k.optimizer),
                SWARM_HUB => Some(&mut k.hub),
                _ => None,
            };
            if let Some(list) = by_block {
                list.push(i);
            }
            let by_role = [
                (p.is_launcher(), &mut k.launcher),
                (p.is_pad(), &mut k.pad),
                (p.is_center(), &mut k.center),
                (p.takes_water(), &mut k.water),
            ];
            for (_, list) in by_role.into_iter().filter(|(is, _)| *is) {
                list.push(i);
            }
        }
        k
    }
}

#[cfg(test)]
mod tests;
