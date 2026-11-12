//! The pumpjack (Milestone 10): a 2 × 3 × 3 processor (`FOOTPRINT`: the well at its front, the motor and crank at its
//! back) that draws crude oil from a reservoir and fills canisters. Placed on the ground, it drills straight down
//! under its base, front cells first, to the first oil sand (or spent rock of an oil deposit), up to [`MAX_DEPTH`]
//! blocks (`sink_well`, at placing); that deposit's draw cap and taper decide its pace like any miner's
//! (`deposits.rs`), so a vein gives about 20 canisters a minute for hours. Oil is never a block in the world.
//!
//! - Each tick it draws `RATE` units a second at full power, `RECOVERY` of it kept, into `Pump::carry`; every
//!   [`CANISTER_UNITS`] kept fills one empty canister from its input (belts bring the empties to its back and take
//!   the full ones from its front). It stops drawing while a canister's worth waits (no empties, or a full output).
//! - Saved for pumpjacks only: the well (deposit and drill bit), the carry, and whether it is `compact`: a pumpjack
//!   from before save 70 was one block, and stays one (taking and giving on every side, drawn a third the size), as
//!   its neighbours leave no room to grow (none of the user's five could). Placed again, it is full size.
//! - Drawn by `view.rs` from `Pump::crank`, which turns while it works (derived, eased so it starts and stops softly).
//!
//! To change its pace: the constants and its spec's power.

use crate::block::{OIL_SAND, PUMPJACK, SPENT_ROCK};
use crate::bytes::{ByteReader, ByteWriter};
use crate::deposits::{DepositKey, Deposits};
use crate::inventory::Stack;
use crate::item::{CRUDE_CANISTER, EMPTY_CANISTER};
use crate::math::IVec3;
use crate::world::World;
use crate::{TICK, TICK_RATE};

use super::super::describe::fmt_int;
use super::super::footprint::{Footprint, Port, Role, Side, Which};
use super::super::power::{FULL_SPEED, NOT_WIRED};
use super::super::Factory;
use super::model::{part, Look, Part};
use super::{Energy, Extra, Pick, ProcessSpec, ProcessTier, Processor, Status};

mod view;

pub(super) use view::draw as draw_jack;

#[cfg(test)]
mod tests;

/// Units a canister holds.
pub const CANISTER_UNITS: f64 = 10.0;
/// Units drawn a second at full power, before the deposit's cap and taper.
const RATE: f64 = 5.0;
/// Share of what is drawn that reaches the canister.
const RECOVERY: f64 = 0.9;
/// How far below its base it searches for oil.
const MAX_DEPTH: i32 = 100;
/// Seconds a turn of the crank takes at full power (a real one strokes 5 to 15 times a minute), and the share of the
/// gap to that pace it closes each tick, so it winds up and runs down over about half a second.
pub const CRANK_SECONDS: f64 = 4.0;
const CRANK_EASE: f64 = 0.06;
/// The first save version with full-size pumpjacks; older ones load `compact`.
pub const FULL_SIZE_SAVED: u32 = 70;

/// Two cells across, three deep (the well at the front), three high; empties in at the back, full canisters out at
/// the front.
pub const FOOTPRINT: Footprint = Footprint {
    size: [2, 3, 3],
    ports: &[
        Port { side: Side::Back, role: Role::In, cell: Which::All },
        Port { side: Side::Front, role: Role::Out, cell: Which::All },
    ],
};

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
    /// A one-block pumpjack from an older save (saved; see the header).
    pub compact: bool,
    /// Smoothed units drawn a second, for the readout (derived).
    pub rate: f64,
    /// The crank's angle in turns, and how far it turned last tick (derived, presentation only: never read by the
    /// core, not saved, so a loaded pumpjack starts from its rest pose).
    pub crank: f64,
    pub spin: f64,
}

impl Pump {
    pub(super) fn write_state(&self, w: &mut ByteWriter) {
        w.bool(self.well.is_some());
        if let Some(well) = self.well {
            well.key.write_state(w);
            w.ivec3(well.bit);
        }
        w.f64(self.carry);
        w.bool(self.compact);
    }

    pub(super) fn read_state(r: &mut ByteReader) -> Option<Pump> {
        let well = if r.bool()? { Some(Well { key: DepositKey::read_state(r)?, bit: r.ivec3()? }) } else { None };
        let carry = r.f64()?;
        let compact = if r.version >= FULL_SIZE_SAVED { r.bool()? } else { true };
        (carry.is_finite() && carry >= 0.0).then_some(Pump { well, carry, compact, ..Pump::default() })
    }
}

pub const PUMPJACK_SPEC: ProcessSpec = ProcessSpec {
    block: PUMPJACK,
    categories: &[],
    pick: Pick::Pump,
    buffers: [1, 0, 1],
    side: 0,
    tiers: &[ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 90 }],
    footprint: FOOTPRINT,
    verb: "Pumping",
    products: "crude oil canisters",
    waiting: "Idle",
    map_colour: 0x3a2e22,
    compute: 0,
    parts: &PARTS,
};

/// The whole machine, still and moving, is drawn by `view.rs` from this part (its box is the footprint).
const PARTS: [Part; 1] = [part([0.0, 0.0, 0.0], [2.0, 3.0, 3.0], Look::Jack)];

impl Processor {
    /// Finds the well below a freshly placed pumpjack (nothing for other processors, or with no oil underneath): the
    /// first oil straight down from its bottom cells, front row first.
    pub(crate) fn sink_well(&mut self, deposits: &mut Deposits, world: &mut World) {
        let base: Vec<IVec3> = self.cells().into_iter().filter(|c| c.y == self.pos.y).collect();
        let Extra::Pump(pump) = &mut self.extra else { return };
        let below = |c: IVec3| (1..=MAX_DEPTH.min(c.y)).map(move |d| c - IVec3::new(0, d, 0));
        pump.well = base.into_iter().flat_map(below).find_map(|bit| {
            let block = world.block_anywhere_or_generate(bit);
            let found = matches!(block, OIL_SAND | SPENT_ROCK).then(|| deposits.lookup(world, bit)).flatten();
            found.filter(|key| key.ore == OIL_SAND).map(|key| Well { key, bit })
        });
    }

    /// Whether this is a one-block pumpjack from an older save.
    pub fn is_compact(&self) -> bool {
        matches!(&self.extra, Extra::Pump(p) if p.compact)
    }

    /// One tick of pumping at `share` of full power (called before `step`), then the crank's turn.
    pub(crate) fn pump(&mut self, deposits: &mut Deposits, world: &mut World, tick: u64, share: u32) {
        self.draw_oil(deposits, world, tick, share);
        let working = self.status == Status::Working;
        let Extra::Pump(pump) = &mut self.extra else { return };
        let pace =
            if working { f64::from(share) / f64::from(FULL_SPEED) / (CRANK_SECONDS * TICK_RATE as f64) } else { 0.0 };
        pump.spin += (pace - pump.spin) * CRANK_EASE;
        if pump.spin < 1e-6 && !working {
            pump.spin = 0.0;
        }
        pump.crank = (pump.crank + pump.spin).fract();
    }

    fn draw_oil(&mut self, deposits: &mut Deposits, world: &mut World, tick: u64, share: u32) {
        let Extra::Pump(pump) = &mut self.extra else { return };
        let Some(well) = pump.well else {
            self.status = Status::NoDeposit;
            return;
        };
        if pump.carry >= CANISTER_UNITS
            && self.input.count(EMPTY_CANISTER) > 0
            && self.out.space_for(CRUDE_CANISTER) > 0
        {
            self.input.remove(EMPTY_CANISTER, 1);
            self.used.push(Stack { item: EMPTY_CANISTER, count: 1 });
            self.out.add(CRUDE_CANISTER, 1);
            self.made.push(Stack { item: CRUDE_CANISTER, count: 1 });
            pump.carry -= CANISTER_UNITS;
        }
        let mut drawn = 0.0;
        self.status = if pump.carry >= CANISTER_UNITS {
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
            pump.carry += drawn * RECOVERY;
            Status::Working
        };
        let rate = drawn / TICK;
        pump.rate = if pump.rate == 0.0 { rate } else { pump.rate + (rate - pump.rate) * (TICK / 2.0) };
    }

    /// Whether it would draw this tick if powered.
    pub(super) fn pump_wants_power(&self) -> bool {
        let Extra::Pump(pump) = &self.extra else { return false };
        pump.well.is_some() && pump.carry < CANISTER_UNITS && self.status != Status::Exhausted
    }

    /// The status line of a pumpjack (`None`: another processor).
    pub(super) fn pump_text(&self) -> Option<String> {
        let Extra::Pump(pump) = &self.extra else { return None };
        Some(match self.status {
            Status::NoDeposit => "No oil below: stand it on the ground right above an oil reservoir".to_string(),
            Status::Exhausted => "The reservoir is dry".to_string(),
            Status::NoPower => NOT_WIRED.to_string(),
            Status::NoInput => "Waiting for empty canisters: a belt brings them in".to_string(),
            Status::OutputFull => "Output full: put a belt leading away, or take the canisters".to_string(),
            _ => format!("Pumping crude oil · {} canisters a minute", (pump.rate * RECOVERY * 6.0).round() as u32),
        })
    }

    /// A line on how much oil is left in the reservoir (empty for other processors).
    pub(super) fn reservoir_line(&self, f: &Factory) -> String {
        let Extra::Pump(pump) = &self.extra else { return String::new() };
        let left = pump.well.and_then(|w| f.deposits.get(&w.key)).map(|s| s.remaining_units() * RECOVERY);
        left.map_or_else(String::new, |units| {
            format!("\nReservoir: {} canisters left", fmt_int((units / CANISTER_UNITS) as u64))
        })
    }
}
