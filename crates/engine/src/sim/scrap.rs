//! What the ruins' rusted blocks leave when broken (`block/ruins.rs`): scrap metal, and now and then a circuit. The
//! amounts draw from the core `rng`, like a leaf's sapling (`saplings.rs`). Called from `Sim::dismantle`.

use super::Sim;
use crate::block::{self, BlockId, RUSTED_GIRDER, RUSTED_MACHINE};
use crate::inventory::Stack;
use crate::item::{CIRCUIT, SCRAP_METAL};

/// Chance a machine husk holds a circuit that survived the rust.
const CIRCUIT_CHANCE: f64 = 0.25;

impl Sim {
    /// The salvage of the rusted block `id`: scrap metal (a husk gives more) and, from a husk, sometimes a circuit.
    /// Empty for any other block.
    pub(crate) fn salvage(&mut self, id: BlockId) -> Vec<Stack> {
        if !block::is_rusted(id) {
            return Vec::new();
        }
        let scrap = match id {
            RUSTED_MACHINE => 2 + self.rng.next_u32() % 3,
            RUSTED_GIRDER => 1 + self.rng.next_u32() % 2,
            _ => 1,
        };
        let mut out = vec![Stack { item: SCRAP_METAL, count: scrap }];
        if id == RUSTED_MACHINE && self.rng.next_f64() < CIRCUIT_CHANCE {
            out.push(Stack { item: CIRCUIT, count: 1 });
        }
        out
    }
}

#[cfg(test)]
mod tests;
