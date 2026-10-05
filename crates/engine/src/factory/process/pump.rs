//! The pumpjack (Milestone 10): a one-block processor that draws crude oil from a reservoir and fills canisters.
//! Placed on the ground, it drills straight down to the first oil sand (or spent rock of an oil deposit) of its
//! column, up to [`MAX_DEPTH`] blocks (`sink_well`, at placing); that deposit's draw cap and taper decide its pace
//! like any miner's (`deposits.rs`), so a vein gives about 20 canisters a minute for hours. Oil is never a block in
//! the world, so there is nothing for the water simulation to flow.
//!
//! - Each tick it draws `RATE` units a second at full power, `RECOVERY` of it kept, into `Pump::carry`; every
//!   [`CANISTER_UNITS`] kept fills one empty canister from its input slot into its output (belts bring the empties
//!   and take the full ones). It stops drawing while a canister's worth waits (no empties, or a full output).
//! - Saved for pumpjacks only: the well (deposit and drill bit) and the carry.
//!
//! To change its pace: the constants and its spec's power.

use crate::block::{tex, OIL_SAND, PUMPJACK, SPENT_ROCK};
use crate::bytes::{ByteReader, ByteWriter};
use crate::deposits::{DepositKey, Deposits};
use crate::item::{CRUDE_CANISTER, EMPTY_CANISTER};
use crate::math::IVec3;
use crate::world::World;
use crate::TICK;

use super::super::describe::fmt_int;
use super::super::footprint::SINGLE;
use super::super::power::{FULL_SPEED, NOT_WIRED};
use super::super::Factory;
use super::model::{part, Look, Part};
use super::{Energy, Pick, ProcessSpec, ProcessTier, Processor, Status};

#[cfg(test)]
mod tests;

/// Units a canister holds.
pub const CANISTER_UNITS: f64 = 10.0;
/// Units drawn a second at full power, before the deposit's cap and taper.
const RATE: f64 = 5.0;
/// Share of what is drawn that reaches the canister.
const RECOVERY: f64 = 0.9;
/// How far below its own block it searches for oil.
const MAX_DEPTH: i32 = 100;

/// Where a pumpjack draws: the reservoir and the block its drill bit reaches (spent rock forms around it).
#[derive(Clone, Copy)]
pub struct Well {
    pub key: DepositKey,
    pub bit: IVec3,
}

#[derive(Default)]
pub struct Pump {
    pub well: Option<Well>,
    /// Oil units kept towards the next canister.
    pub carry: f64,
    /// Smoothed units drawn a second, for the readout (derived).
    pub rate: f64,
}

impl Pump {
    pub(super) fn write_state(&self, w: &mut ByteWriter) {
        w.bool(self.well.is_some());
        if let Some(well) = self.well {
            well.key.write_state(w);
            w.ivec3(well.bit);
        }
        w.f64(self.carry);
    }

    pub(super) fn read_state(r: &mut ByteReader) -> Option<Pump> {
        let well = if r.bool()? { Some(Well { key: DepositKey::read_state(r)?, bit: r.ivec3()? }) } else { None };
        let carry = r.f64()?;
        (carry.is_finite() && carry >= 0.0).then_some(Pump { well, carry, rate: 0.0 })
    }
}

pub const PUMPJACK_SPEC: ProcessSpec = ProcessSpec {
    block: PUMPJACK,
    categories: &[],
    pick: Pick::Pump,
    buffers: [1, 0, 1],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 90 }],
    footprint: SINGLE,
    verb: "Pumping",
    products: "crude oil canisters",
    waiting: "Idle",
    map_colour: 0x3a2e22,
    parts: &PARTS,
};

const BODY: [u16; 3] = [tex::PUMPJACK_TOP, tex::PUMPJACK_SIDE, tex::FRAME];
const MAST: Look = Look::Tex([tex::PUMPJACK_SIDE; 3]);

/// A banded plinth under an olive housing with a well-head on the front, an A-frame carrying a walking beam that
/// rocks while it pumps (a head at the front, a counterweight behind) and a status lamp.
const PARTS: [Part; 9] = [
    part([0.0, -0.42, 0.0], [0.94, 0.16, 0.94], Look::Band(tex::FRAME)),
    part([0.0, -0.12, -0.04], [0.8, 0.44, 0.7], Look::Tex(BODY)),
    part([0.0, -0.1, 0.4], [0.22, 0.3, 0.12], Look::Tex([tex::FRAME; 3])),
    part([-0.3, 0.36, -0.04], [0.1, 0.5, 0.1], MAST),
    part([0.3, 0.36, -0.04], [0.1, 0.5, 0.1], MAST),
    part([0.0, 0.64, -0.04], [0.2, 0.1, 0.9], Look::Press(0.4, [tex::STEEL; 3])),
    part([0.0, 0.5, 0.42], [0.16, 0.3, 0.1], Look::Press(1.0, [tex::FRAME; 3])),
    part([0.0, 0.56, -0.4], [0.3, 0.22, 0.14], Look::Press(-0.5, [tex::FRAME; 3])),
    part([0.33, 0.14, 0.36], [0.12, 0.08, 0.12], Look::Lamp),
];

impl Processor {
    /// Finds the well below a freshly placed pumpjack (nothing for other processors, or with no oil underneath).
    pub(crate) fn sink_well(&mut self, deposits: &mut Deposits, world: &mut World) {
        if self.spec.pick != Pick::Pump {
            return;
        }
        let pos = self.pos;
        self.pump.well = (1..=MAX_DEPTH.min(pos.y)).map(|d| pos - IVec3::new(0, d, 0)).find_map(|bit| {
            let block = world.block_anywhere_or_generate(bit);
            let found = matches!(block, OIL_SAND | SPENT_ROCK).then(|| deposits.lookup(world, bit)).flatten();
            found.filter(|key| key.ore == OIL_SAND).map(|key| Well { key, bit })
        });
    }

    /// One tick of pumping at `share` of full power (called before `step`).
    pub(crate) fn pump(&mut self, deposits: &mut Deposits, world: &mut World, tick: u64, share: u32) {
        if self.spec.pick != Pick::Pump {
            return;
        }
        let Some(well) = self.pump.well else {
            self.status = Status::NoDeposit;
            return;
        };
        if self.pump.carry >= CANISTER_UNITS
            && self.input.count(EMPTY_CANISTER) > 0
            && self.out.space_for(CRUDE_CANISTER) > 0
        {
            self.input.remove(EMPTY_CANISTER, 1);
            self.out.add(CRUDE_CANISTER, 1);
            self.pump.carry -= CANISTER_UNITS;
        }
        let mut drawn = 0.0;
        self.status = if self.pump.carry >= CANISTER_UNITS {
            if self.out.space_for(CRUDE_CANISTER) == 0 {
                Status::OutputFull
            } else {
                Status::NoInput
            }
        } else if deposits.get(&well.key).is_none_or(|s| s.exhausted()) {
            Status::Exhausted
        } else if share == 0 {
            Status::NoPower
        } else {
            drawn = deposits.draw(world, &well.key, RATE * share as f64 / FULL_SPEED as f64, well.bit, tick, TICK);
            self.pump.carry += drawn * RECOVERY;
            Status::Working
        };
        let rate = drawn / TICK;
        self.pump.rate =
            if self.pump.rate == 0.0 { rate } else { self.pump.rate + (rate - self.pump.rate) * (TICK / 2.0) };
    }

    /// Whether it would draw this tick if powered.
    pub(super) fn pump_wants_power(&self) -> bool {
        self.pump.well.is_some() && self.pump.carry < CANISTER_UNITS && self.status != Status::Exhausted
    }

    /// The status line of a pumpjack (`None`: another processor).
    pub(super) fn pump_text(&self) -> Option<String> {
        (self.spec.pick == Pick::Pump).then(|| match self.status {
            Status::NoDeposit => "No oil below: stand it on the ground right above an oil reservoir".to_string(),
            Status::Exhausted => "The reservoir is dry".to_string(),
            Status::NoPower => NOT_WIRED.to_string(),
            Status::NoInput => "Waiting for empty canisters: a belt brings them in".to_string(),
            Status::OutputFull => "Output full: put a belt leading away, or take the canisters".to_string(),
            _ => format!("Pumping crude oil · {} canisters a minute", (self.pump.rate * RECOVERY * 6.0).round() as u32),
        })
    }

    /// A line on how much oil is left in the reservoir (empty for other processors).
    pub(super) fn reservoir_line(&self, f: &Factory) -> String {
        let left = self.pump.well.and_then(|w| f.deposits.get(&w.key)).map(|s| s.remaining_units() * RECOVERY);
        left.map_or_else(String::new, |units| {
            format!("\nReservoir: {} canisters left", fmt_int((units / CANISTER_UNITS) as u64))
        })
    }
}
