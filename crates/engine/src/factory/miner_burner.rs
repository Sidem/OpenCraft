//! The coal miner's fire: a miner at tier [`COAL_TIER`] drills on fuel, not on a grid. It holds one stack of fuel
//! (anything `recipes::fuel_energy` knows) and the energy of the item it lit, in kW·ticks like a generator
//! (1 kJ = `TICK_RATE`). It burns only on ticks it can drill, so fuel lasts exactly as long as the work does.
//!
//! Fuel comes in by hand (the panel), from a belt (`links.rs`: a coal miner is a sink for fuel; miners and quarries
//! beside it do not feed it) or from its own digging: a coal miner on coal keeps [`SELF_FUEL_RESERVE`] coal back before
//! anything goes out. It takes no wire (`wiring.rs`).
//! Its numbers are [`COAL_STATS`], one row beside `MINER_TIERS`; it has no kit and no upgrade. To tune it: that row.

use crate::block::{tex, BlockId, COAL_MINER, MINER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::item::{self, ItemId};
use crate::recipes::fuel_energy;
use crate::TICK_RATE;

use super::buffer::Buffer;
use super::miner::{Miner, MinerStatus, MinerTier, MINER_TIERS};
use super::panel::{Panel, ROLE_FUEL, ROLE_OUTPUT};

/// The tier index a coal miner saves: one past the electric tiers.
pub const COAL_TIER: u8 = MINER_TIERS.len() as u8;

/// Slow and wasteful: it draws half a Mk1's units a second and keeps 40% of them (a third of a Mk1's ore, from a
/// deposit that runs down half as fast again per ore), for 10 kW of fire. One coal lasts 27 s: about five coal dug.
pub const COAL_STATS: MinerTier = MinerTier { rate: 0.5, recovery: 0.4, power: 10, side: tex::SMELTER_SIDE };

/// Fuel a coal miner on fuel keeps back from its own digging, in items.
const SELF_FUEL_RESERVE: u32 = 3;

pub struct Burner {
    /// One slot for a coal miner, none for an electric miner.
    pub fuel: Buffer,
    /// Energy left from the fuel it lit, in kW·ticks.
    pub energy: u32,
}

impl Burner {
    pub fn new(coal: bool) -> Burner {
        Burner { fuel: Buffer::new(coal as usize), energy: 0 }
    }

    /// How many of `item` it would take now: fuel only, up to the stack's room.
    pub fn room_for(&self, item: ItemId) -> u32 {
        if fuel_energy(item).is_some() {
            self.fuel.space_for(item)
        } else {
            0
        }
    }

    /// Whole kJ it holds.
    pub fn stored_kj(&self) -> u32 {
        self.energy / TICK_RATE
    }

    /// Burns `kw` for one tick if `wanted`, lighting the next item when what is left cannot cover it.
    /// False when it should have burned and had no fuel.
    pub fn fire(&mut self, wanted: bool, kw: u32) -> bool {
        if !wanted {
            return false;
        }
        if self.energy < kw {
            let Some(slot) = self.fuel.slots.iter().position(|s| !s.is_empty()) else { return false };
            self.energy += fuel_energy(self.fuel.slots[slot].item).unwrap_or(0) * TICK_RATE;
            self.fuel.take(slot, 1);
        }
        let burned = self.energy >= kw;
        if burned {
            self.energy -= kw;
        }
        burned
    }

    /// Of `whole` freshly dug `ore`, how many it keeps as fuel (and takes), topping up to [`SELF_FUEL_RESERVE`].
    pub fn keep_own(&mut self, ore: ItemId, whole: u32) -> u32 {
        let put = whole.min(SELF_FUEL_RESERVE.saturating_sub(self.fuel.total())).min(self.room_for(ore));
        self.fuel.add(ore, put);
        put
    }

    pub fn write_state(&self, w: &mut ByteWriter) {
        self.fuel.write_state(w);
        w.u32(self.energy);
    }

    pub fn read_state(r: &mut ByteReader) -> Option<Burner> {
        let fuel = Buffer::read_state(r, 1)?;
        Some(Burner { fuel, energy: r.u32()? })
    }

    /// What it holds, as a readout line ("Fuel: 3 Coal Ore · 270 kJ stored"), if anything.
    pub fn line(&self) -> Option<String> {
        let s = self.fuel.slots.first().filter(|s| !s.is_empty());
        let mut parts = s.map(|s| format!("Fuel: {} {}", s.count, item::name(s.item))).into_iter().collect::<Vec<_>>();
        if self.energy > 0 {
            parts.push(format!("{} kJ stored", self.stored_kj()));
        }
        (!parts.is_empty()).then(|| parts.join(" · "))
    }
}

impl Miner {
    /// Whether it burns fuel instead of drawing power.
    pub fn is_coal(&self) -> bool {
        self.tier == COAL_TIER
    }

    /// The block it stands as (a coal miner has its own; the electric tiers share `MINER`).
    pub fn block(&self) -> BlockId {
        if self.is_coal() {
            COAL_MINER
        } else {
            MINER
        }
    }

    /// The panel of a coal miner: its fuel and its output.
    pub fn panel(&self, status: String) -> Option<Panel> {
        self.is_coal().then(|| {
            let burning = self.status == MinerStatus::Running;
            Panel {
                block: COAL_MINER,
                recipe: None,
                choosable: false,
                progress: 0,
                fire: if burning { self.burner.energy.div_ceil(self.stats().power * TICK_RATE) } else { 0 },
                slots: vec![(ROLE_FUEL, self.burner.fuel.slots[0]), (ROLE_OUTPUT, self.out.slots[0])],
                status,
                filter: None,
            }
        })
    }
}

#[cfg(test)]
mod tests;
