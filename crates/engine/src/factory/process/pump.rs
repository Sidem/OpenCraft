//! The pumpjack (Milestone 10): a 2 × 3 × 3 processor (`FOOTPRINT`: the well at its front, the motor and crank at its
//! back) that draws crude oil from a reservoir and fills barrels. Placed on the ground, it drills straight down
//! under its base, front cells first, to the first oil sand (or spent rock of an oil deposit), up to [`MAX_DEPTH`]
//! blocks (`sink_well`, at placing); that deposit's draw cap and taper decide its pace like any miner's
//! (`deposits.rs`), so a vein gives about 20 barrels a minute for hours. Oil is never a block in the world.
//!
//! - Each tick it draws its tier's rate (`JACK_TIERS`: units a second at full power, and the share kept) into `Pump::carry`; every
//!   [`BARREL_UNITS`] kept fills one empty barrel from its input (belts bring the empties to its back and take
//!   the full ones from its front). It stops drawing while a barrel's worth waits (no empties, or a full output).
//! - Saved for pumpjacks only: the well (deposit and drill bit), the carry, and whether it is `compact`: a pumpjack
//!   from before save 70 was one block, and stays one (taking and giving on every side, drawn a third the size), as
//!   its neighbours leave no room to grow (none of the user's five could). Placed again, it is full size.
//! - Drawn by `view.rs` from `Pump::crank`, which turns while it works (derived, eased so it starts and stops softly).
//!
//! Mk2 (a gold kit, `factory/tiers.rs`) is `JACK_TIERS[1]` and the spec's second tier. To change its pace: the constants and
//! the spec's power.

use crate::block::{tex, BlockId, GAS_SHALE, GAS_WELL, OIL_SAND, PUMPJACK, SPENT_ROCK, WELLHEAD_SEPARATOR};
use crate::bytes::{ByteReader, ByteWriter};
use crate::deposits::{DepositKey, Deposits};
use crate::inventory::Stack;
use crate::item::{ItemId, CRUDE_BARREL, EMPTY_BARREL, RAW_GAS_BARREL};
use crate::math::{hash3, IVec3};
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

/// Units a barrel holds.
pub const BARREL_UNITS: f64 = 10.0;
/// What a pumpjack tier draws, Mk1 first (the spec's tiers, `PUMPJACK_SPEC`, hold the power).
pub struct JackTier {
    /// Units drawn a second at full power, before the deposit's cap and taper.
    pub rate: f64,
    /// Share of what is drawn that reaches the barrel.
    pub recovery: f64,
}

/// Mk2 is the gold kit's (`factory/tiers.rs`): twice the draw, which a lode's higher cap can use, and a miner Mk5's recovery.
pub const JACK_TIERS: [JackTier; 2] = [JackTier { rate: 5.0, recovery: 0.9 }, JackTier { rate: 10.0, recovery: 0.97 }];
/// How far below its base it searches for oil.
const MAX_DEPTH: i32 = 100;
/// Seconds a turn of the crank takes at full power (a real one strokes 5 to 15 times a minute), and the share of the
/// gap to that pace it closes each tick, so it winds up and runs down over about half a second.
pub const CRANK_SECONDS: f64 = 4.0;
const CRANK_EASE: f64 = 0.06;
/// The first save version with full-size pumpjacks; older ones load `compact`.
pub const FULL_SIZE_SAVED: u32 = 70;

/// Two cells across, three deep (the well at the front), three high; empties in at the back, full barrels out at
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
    /// Oil units kept towards the next barrel.
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
    tiers: &[
        ProcessTier { energy: Energy::Electric, speed: 1000, fuel: 0, power: 90 },
        ProcessTier { energy: Energy::Electric, speed: 2000, fuel: 0, power: 160 },
    ],
    footprint: FOOTPRINT,
    verb: "Pumping",
    products: "crude oil barrels",
    waiting: "Idle",
    map_colour: 0x3a2e22,
    compute: 0,
    parts: &PARTS,
};

/// The whole machine, still and moving, is drawn by `view.rs` from this part (its box is the footprint).
///
/// The slab under it is the tier's band (`Look::Band`: the Mk studs on its sides, its stripe in the kit's colour), a hair
/// outside the concrete pad `view.rs` lays; a one-block pumpjack from an old save has no room for it (`model.rs`).
const PARTS: [Part; 2] = [
    part([0.0, -1.44, 0.0], [1.9, 0.12, 2.9], Look::Band(tex::CONCRETE)),
    part([0.0, 0.0, 0.0], [2.0, 3.0, 3.0], Look::Jack),
];

/// Deterministic associated gas-to-oil ratio for an oil deposit: 1 raw gas barrel per `2..=4` crude oil barrels.
pub fn associated_gas_ratio(key: DepositKey) -> u32 {
    let h = hash3(key.cx as u32 ^ 0xA550, key.cz, i32::from(key.index), key.tier as i32);
    2 + (h % 3)
}

impl Processor {
    fn well_ore(&self) -> BlockId {
        if self.spec.block == GAS_WELL {
            GAS_SHALE
        } else {
            OIL_SAND
        }
    }

    fn well_barrel(&self) -> ItemId {
        if self.spec.block == GAS_WELL {
            RAW_GAS_BARREL
        } else {
            CRUDE_BARREL
        }
    }

    fn is_separator(&self) -> bool {
        self.spec.block == WELLHEAD_SEPARATOR
    }

    /// Finds the well below a freshly placed pumpjack, gas well or wellhead separator: the first matching deposit
    /// block straight down from its bottom cells, front row first.
    pub(crate) fn sink_well(&mut self, deposits: &mut Deposits, world: &mut World) {
        let want = self.well_ore();
        let base: Vec<IVec3> = self.cells().into_iter().filter(|c| c.y == self.pos.y).collect();
        let Extra::Pump(pump) = &mut self.extra else { return };
        let below = |c: IVec3| (1..=MAX_DEPTH.min(c.y)).map(move |d| c - IVec3::new(0, d, 0));
        pump.well = base.into_iter().flat_map(below).find_map(|bit| {
            let block = world.block_anywhere_or_generate(bit);
            let found = (block == want || block == SPENT_ROCK).then(|| deposits.lookup(world, bit)).flatten();
            found.filter(|key| key.ore == want).map(|key| Well { key, bit })
        });
    }

    /// Whether this is a one-block pumpjack from an older save.
    pub fn is_compact(&self) -> bool {
        matches!(&self.extra, Extra::Pump(p) if p.compact)
    }

    /// What this well's tier draws.
    fn jack(&self) -> &'static JackTier {
        &JACK_TIERS[usize::from(self.tier).min(JACK_TIERS.len() - 1)]
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
        let jack = self.jack();
        let sep = self.is_separator();
        let product = self.well_barrel();
        let Extra::Pump(pump) = &mut self.extra else { return };
        let Some(well) = pump.well else {
            self.status = Status::NoDeposit;
            return;
        };
        let (need_units, need_empties, main_count, gas_count) = if sep {
            let r = associated_gas_ratio(well.key);
            (f64::from(r) * BARREL_UNITS, r + 1, r, 1u32)
        } else {
            (BARREL_UNITS, 1, 1, 0)
        };
        let side_room = gas_count == 0
            || (self.side.count(RAW_GAS_BARREL) < super::SIDE_ROOM && self.side.space_for(RAW_GAS_BARREL) >= gas_count);
        if pump.carry >= need_units
            && self.input.count(EMPTY_BARREL) >= need_empties
            && self.out.space_for(product) >= main_count
            && side_room
        {
            self.input.remove(EMPTY_BARREL, need_empties);
            self.used.push(Stack { item: EMPTY_BARREL, count: need_empties });
            self.out.add(product, main_count);
            self.made.push(Stack { item: product, count: main_count });
            if gas_count > 0 {
                self.side.add(RAW_GAS_BARREL, gas_count);
                self.made.push(Stack { item: RAW_GAS_BARREL, count: gas_count });
            }
            pump.carry -= need_units;
        }
        let mut drawn = 0.0;
        self.status = if pump.carry >= need_units {
            if self.out.space_for(product) < main_count || !side_room {
                Status::OutputFull
            } else {
                Status::NoInput
            }
        } else if deposits.get(&well.key).is_none_or(|s| s.exhausted()) {
            Status::Exhausted
        } else if share == 0 {
            Status::NoPower
        } else {
            drawn = deposits.draw(world, &well.key, jack.rate * share as f64 / FULL_SPEED as f64, well.bit, tick, TICK);
            pump.carry += drawn * jack.recovery;
            Status::Working
        };
        let rate = drawn / TICK;
        pump.rate = if pump.rate == 0.0 { rate } else { pump.rate + (rate - pump.rate) * (TICK / 2.0) };
    }

    /// Whether it would draw this tick if powered.
    pub(super) fn pump_wants_power(&self) -> bool {
        let Extra::Pump(pump) = &self.extra else { return false };
        let threshold = if self.is_separator() {
            pump.well.map_or(BARREL_UNITS, |w| f64::from(associated_gas_ratio(w.key)) * BARREL_UNITS)
        } else {
            BARREL_UNITS
        };
        pump.well.is_some() && pump.carry < threshold && self.status != Status::Exhausted
    }

    /// The status line of a pumpjack, gas well or wellhead separator (`None`: another processor).
    pub(super) fn pump_text(&self) -> Option<String> {
        let Extra::Pump(pump) = &self.extra else { return None };
        let is_gas = self.spec.block == GAS_WELL;
        let is_sep = self.is_separator();
        Some(match self.status {
            Status::NoDeposit if is_gas => {
                "No gas below: stand it on the ground right above a gas shale field".to_string()
            }
            Status::NoDeposit => "No oil below: stand it on the ground right above an oil reservoir".to_string(),
            Status::Exhausted => "The reservoir is dry".to_string(),
            Status::NoPower => NOT_WIRED.to_string(),
            Status::NoInput => "Waiting for empty barrels: a belt brings them in".to_string(),
            Status::OutputFull if is_sep && self.side.count(RAW_GAS_BARREL) >= super::SIDE_ROOM => {
                "Raw gas has nowhere to go: put a belt leading away from the side port, or take it".to_string()
            }
            Status::OutputFull => "Output full: put a belt leading away, or take the barrels".to_string(),
            _ if is_gas => {
                format!(
                    "Drawing raw gas · {} barrels a minute",
                    (pump.rate * self.jack().recovery * 6.0).round() as u32
                )
            }
            _ if is_sep => {
                let ratio = pump.well.map_or(3, |w| associated_gas_ratio(w.key));
                format!(
                    "Separating crude & gas · {} barrels a minute (1 gas per {ratio} crude)",
                    (pump.rate * self.jack().recovery * 6.0).round() as u32
                )
            }
            _ => {
                format!(
                    "Pumping crude oil · {} barrels a minute",
                    (pump.rate * self.jack().recovery * 6.0).round() as u32
                )
            }
        })
    }

    /// A line on how much reserve is left in the reservoir (empty for other processors).
    pub(super) fn reservoir_line(&self, f: &Factory) -> String {
        let Extra::Pump(pump) = &self.extra else { return String::new() };
        let Some(well) = pump.well else { return String::new() };
        let left = f.deposits.get(&well.key).map(|s| s.remaining_units() * self.jack().recovery);
        left.map_or_else(String::new, |units| {
            let extra = if self.is_separator() {
                format!(" · associated gas 1:{}", associated_gas_ratio(well.key))
            } else {
                String::new()
            };
            format!("\nReservoir: {} barrels left{extra}", fmt_int((units / BARREL_UNITS) as u64))
        })
    }
}
