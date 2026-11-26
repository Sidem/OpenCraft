//! The space elevator, phase 1 (M14A; the user's picks of 2026-11-26, `docs/DECISIONS.md`). One per world: an Elevator
//! Anchor stands within [`TOWN_REACH`] blocks of the depot of a town grown to [`MIN_BUILT`] stages (a town or a city),
//! and the tether climbs from it along "up" in [`SECTIONS`] sections of [`SECTION_HEIGHT`] blocks.
//!
//! - **Orders** ([`ORDERS`]): each section is a large order delivered to the anchor's store (belts into its back and
//!   sides); a complete order raises the tether one section.
//! - **The draw** ([`DRAW`]): from the first section on, while the next one climbs, its climbers use up a minute of
//!   goods every minute, or the elevator stalls (its order waits) until they are there. A standing order: it pauses,
//!   never takes back, and a finished phase draws nothing. The empty barrels of the rocket fuel come out of the front.
//! - **Its town**: while the town it stands by is stalled or asleep, the elevator waits too.
//! - **Breaking the anchor** puts it to sleep (sections and deliveries kept); an anchor put back on its spot wakes it.
//!
//! Core state on `Sim` (save 89), changed only in `Sim::step` (`sync_elevator` before the tick's actions,
//! `step_elevator` after the towns). The tether is drawn by `view.rs` (presentation only).
//! To tune: the tables below. A section's order is saved one count per line, so changing a count is safe; adding or
//! removing a line needs a save version.

use crate::block::{self, CONCRETE};
use crate::bytes::{ByteReader, ByteWriter};
use crate::factory::Want;
use crate::item::{ItemId, ALUMINIUM_PLATE, CIRCUIT, EMPTY_BARREL, PLASTIC, PROCESSOR, ROCKET_FUEL_BARREL, STEEL_BEAM};
use crate::math::IVec3;
use crate::settlements::{deliver, take_upkeep, Line, MINUTE};
use crate::sim::{Sim, SimEvent};

mod view;

/// Sections in phase 1; the tether is this many sections tall when it is done.
pub const SECTIONS: usize = 8;
/// Blocks of tether each section adds.
pub const SECTION_HEIGHT: i32 = 64;
/// Stages a town must have built for an anchor to stand by it (4: a town; 5: a city).
pub const MIN_BUILT: u8 = 4;
/// How far (horizontally, in blocks) from that town's depot the anchor may stand.
pub const TOWN_REACH: i32 = 64;
/// Minutes of the draw the anchor's store stocks.
const STOCK_MINUTES: u32 = 2;

const fn b(id: block::BlockId) -> ItemId {
    ItemId::block(id)
}

/// The order that raises each section, by sections built (each about 1.5 to 3 times a city's order).
pub const ORDERS: [&[Line]; SECTIONS] = [
    &[(b(CONCRETE), 12000), (STEEL_BEAM, 8000), (ALUMINIUM_PLATE, 2000), (CIRCUIT, 2000)],
    &[(b(CONCRETE), 8000), (STEEL_BEAM, 10000), (ALUMINIUM_PLATE, 4000), (PLASTIC, 4000), (CIRCUIT, 2000)],
    &[(STEEL_BEAM, 12000), (ALUMINIUM_PLATE, 6000), (PLASTIC, 6000), (CIRCUIT, 3000), (PROCESSOR, 1000)],
    &[(STEEL_BEAM, 12000), (ALUMINIUM_PLATE, 8000), (PLASTIC, 8000), (PROCESSOR, 2000), (ROCKET_FUEL_BARREL, 2000)],
    &[(STEEL_BEAM, 14000), (ALUMINIUM_PLATE, 10000), (PLASTIC, 8000), (PROCESSOR, 2000), (ROCKET_FUEL_BARREL, 2000)],
    &[(STEEL_BEAM, 16000), (ALUMINIUM_PLATE, 12000), (PLASTIC, 8000), (PROCESSOR, 3000), (ROCKET_FUEL_BARREL, 3000)],
    &[(STEEL_BEAM, 16000), (ALUMINIUM_PLATE, 14000), (PLASTIC, 10000), (PROCESSOR, 3000), (ROCKET_FUEL_BARREL, 3000)],
    &[(STEEL_BEAM, 16000), (ALUMINIUM_PLATE, 16000), (PLASTIC, 10000), (PROCESSOR, 4000), (ROCKET_FUEL_BARREL, 4000)],
];

/// What the climbers use up a minute while a section above the first climbs.
pub const DRAW: &[Line] =
    &[(STEEL_BEAM, 60), (ALUMINIUM_PLATE, 40), (PLASTIC, 30), (CIRCUIT, 10), (ROCKET_FUEL_BARREL, 10)];

/// Why an anchor can't stand somewhere.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unfit {
    /// The world's elevator stands elsewhere (or sleeps elsewhere: put its anchor back on its spot).
    Taken,
    /// No town of [`MIN_BUILT`] stages has its depot within [`TOWN_REACH`] blocks.
    NoTown,
}

impl Unfit {
    /// What the placement hint says.
    pub fn text(self) -> &'static str {
        match self {
            Unfit::Taken => "The world's space elevator stands elsewhere",
            Unfit::NoTown => "An elevator anchor stands near the depot of a town or a city",
        }
    }
}

/// The world's elevator: where its anchor stands, the town it stands by and how far it has climbed.
#[derive(Clone, PartialEq, Debug)]
pub struct Elevator {
    /// The anchor's position (its machine's anchor cell).
    pub pos: IVec3,
    /// The town it stands by (an index into `Settlements::towns`, which never shrinks).
    pub town: u16,
    /// Sections built.
    pub built: u8,
    /// One count per line of the open order (none once phase 1 is done).
    pub delivered: Vec<u32>,
    /// A minute of the draw is missing: the order waits.
    pub stalled: bool,
    /// The tick the next minute of the draw is due.
    pub due: u64,
    /// Its anchor is broken.
    pub dormant: bool,
}

impl Elevator {
    fn new(pos: IVec3, town: u16) -> Elevator {
        Elevator { pos, town, built: 0, delivered: vec![0; ORDERS[0].len()], stalled: false, due: 0, dormant: false }
    }

    /// The open order (empty once phase 1 is done).
    pub fn order(&self) -> &'static [Line] {
        ORDERS.get(self.built as usize).copied().unwrap_or(&[])
    }

    /// Whether phase 1 is done.
    pub fn done(&self) -> bool {
        self.built as usize >= SECTIONS
    }

    /// What the climbers use up a minute now: the draw while a section above the first climbs, else nothing.
    pub fn draw(&self) -> &'static [Line] {
        if self.built == 0 || self.done() {
            &[]
        } else {
            DRAW
        }
    }

    /// What the anchor takes in, and up to how many of each: [`STOCK_MINUTES`] of the draw, plus (unless stalled) what
    /// the open order still misses.
    pub fn wants(&self) -> Vec<Line> {
        let mut wants: Vec<Line> = self.draw().iter().map(|&(i, n)| (i, n * STOCK_MINUTES)).collect();
        if self.stalled {
            return wants;
        }
        for (&(item, want), &got) in self.order().iter().zip(&self.delivered) {
            match wants.iter_mut().find(|w| w.0 == item) {
                Some(w) => w.1 += want.saturating_sub(got),
                None if got < want => wants.push((item, want - got)),
                None => {}
            }
        }
        wants
    }

    /// How far the open section has come, 0 to 1 (by items delivered).
    pub fn progress(&self) -> f32 {
        let want: u32 = self.order().iter().map(|l| l.1).sum();
        let got: u32 = self.delivered.iter().sum();
        if want == 0 {
            0.0
        } else {
            got as f32 / want as f32
        }
    }

    pub fn write(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u16(self.town);
        w.u8(self.built);
        w.count(self.delivered.len());
        self.delivered.iter().for_each(|&d| w.u32(d));
        w.bool(self.stalled);
        w.u64(self.due);
        w.bool(self.dormant);
    }

    /// Reads what `write` wrote; `None` when the sections or deliveries don't fit the orders.
    pub fn read(r: &mut ByteReader) -> Option<Elevator> {
        let (pos, town, built) = (r.ivec3()?, r.u16()?, r.u8()?);
        let delivered = (0..r.count()?).map(|_| r.u32()).collect::<Option<Vec<u32>>>()?;
        let e = Elevator { pos, town, built, delivered, stalled: r.bool()?, due: r.u64()?, dormant: r.bool()? };
        (e.built as usize <= SECTIONS && e.delivered.len() == e.order().len()).then_some(e)
    }
}

/// Writes the world's elevator, if it has one (`Sim::write_state`).
pub fn write_state(e: &Option<Elevator>, w: &mut ByteWriter) {
    w.bool(e.is_some());
    if let Some(e) = e {
        e.write(w);
    }
}

/// Reads what `write_state` wrote.
pub fn read_state(r: &mut ByteReader) -> Option<Option<Elevator>> {
    Some(if r.bool()? { Some(Elevator::read(r)?) } else { None })
}

impl Sim {
    /// Whether an anchor may stand at `pos`: the town it would stand by, or why not. A pure function of the towns and
    /// the elevator, so the placement hint and the core agree.
    pub fn elevator_site(&self, pos: IVec3) -> Result<u16, Unfit> {
        if let Some(e) = &self.elevator {
            return if e.dormant && e.pos == pos { Ok(e.town) } else { Err(Unfit::Taken) };
        }
        let near = |t: &crate::settlements::Town| {
            let (dx, dz) = (i64::from(t.pos.x - pos.x), i64::from(t.pos.z - pos.z));
            dx * dx + dz * dz <= i64::from(TOWN_REACH) * i64::from(TOWN_REACH)
        };
        let towns = &self.settlements.towns;
        let town = towns.iter().position(|t| !t.dormant && t.built >= MIN_BUILT && near(t));
        town.map(|i| i as u16).ok_or(Unfit::NoTown)
    }

    /// Stands the elevator's anchor at `pos` by `town` (from `elevator_site`), waking a sleeping one.
    pub(crate) fn place_elevator(&mut self, pos: IVec3, town: u16) {
        match &mut self.elevator {
            Some(e) => e.dormant = false,
            None => self.elevator = Some(Elevator::new(pos, town)),
        }
    }

    /// Its anchor at `pos` is broken: the elevator sleeps.
    pub(crate) fn elevator_broken(&mut self, pos: IVec3) {
        if let Some(e) = self.elevator.as_mut().filter(|e| e.pos == pos) {
            e.dormant = true;
        }
    }

    /// Tells the anchor what it takes in now.
    pub(crate) fn sync_elevator(&mut self) {
        let Some(e) = &self.elevator else { return };
        let wants = e.wants().into_iter().map(|(item, cap)| Want { item, cap, perk: false }).collect();
        self.factory.set_depot_wants(e.pos, wants);
    }

    /// One tick: the draw when due, then deliveries toward the open section unless stalled; a complete order raises
    /// the tether a section. Waits while asleep, done, or while its town is stalled or asleep.
    pub(crate) fn step_elevator(&mut self) {
        let Some(e) = self.elevator.as_mut() else { return };
        let town = self.settlements.towns.get(e.town as usize);
        if e.dormant || e.done() || town.is_none_or(|t| t.stalled || t.dormant) {
            return;
        }
        let Some(store) = self.factory.depot_store(e.pos) else { return };
        let from = self.events.len();
        let draw = e.draw();
        if !draw.is_empty() && self.tick >= e.due {
            e.stalled = !take_upkeep(store, draw, &mut self.events);
            if !e.stalled {
                e.due = self.tick + MINUTE;
            }
        }
        if !e.stalled && deliver(e.order(), &mut e.delivered, draw, store, &mut self.events) {
            e.built += 1;
            e.delivered = vec![0; e.order().len()];
            e.due = self.tick + MINUTE;
            self.events.push(SimEvent::ElevatorRose { pos: e.pos, built: e.built });
        }
        let barrels: u32 = self.events[from..]
            .iter()
            .map(|ev| match ev {
                SimEvent::Consumed { item, count } if *item == ROCKET_FUEL_BARREL => *count,
                _ => 0,
            })
            .sum();
        if let Some((_, side)) = self.factory.depot_perk(e.pos).filter(|_| barrels > 0) {
            side.add(EMPTY_BARREL, barrels);
        }
    }
}

#[cfg(test)]
mod tests;
