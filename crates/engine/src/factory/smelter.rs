//! Smelter: melts ore into ingots while it has fuel. Three buffers (slots each from `MACHINES`): ore
//! in `input`, fuel in `fuel`, ingots in `out`. Belts and miners deliver into it and it sorts what
//! arrives (fuel from `recipes::FUELS`, ore any smelter recipe uses; anything else is refused). The
//! recipe follows the ore in `input` (`recipes::MACHINE_RECIPES`). Like a box, it pushes one ingot a
//! tick into the next belt leading away (round-robin). Right-click opens its panel (`panel.rs`),
//! where the player can put ore and fuel in and take the ingots.
//!
//! Invariants: a batch uses up its inputs when it starts and only starts when its output fits. Work
//! and fire are counted in whole ticks, so batches finish on exact tick boundaries. The fire burns
//! only while a batch is in progress; a new fuel item is lit when it runs out.

use crate::block::{tex, SMELTER};
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::{self, ItemId};
use crate::math::{IVec3, Vec3};
use crate::recipes::{burn_time, machine_recipe_using, MachineRecipe, MACHINE_RECIPES};
use crate::TICK_RATE;

use super::belt::Belt;
use super::buffer::Buffer;
use super::panel::{Panel, ROLE_FUEL, ROLE_INPUT, ROLE_OUTPUT};
use super::render::push_box;
use super::{ticks, Factory, Kind, Machine};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SmelterStatus {
    Working,
    NoOre,
    NoFuel,
    OutputFull,
}

/// Every status, in declaration order: saves store `status as u8`.
const STATUSES: [SmelterStatus; 4] =
    [SmelterStatus::Working, SmelterStatus::NoOre, SmelterStatus::NoFuel, SmelterStatus::OutputFull];

pub struct Smelter {
    pub pos: IVec3,
    pub input: Buffer,
    pub fuel: Buffer,
    pub out: Buffer,
    /// The batch in progress (index into `MACHINE_RECIPES`) and the ticks of work it has had.
    pub batch: Option<u16>,
    pub progress: u32,
    /// Ticks of work left in the fire.
    pub burn: u32,
    /// Belt indices leading away from it.
    pub outs: Vec<u32>,
    pub next_out: usize,
    pub status: SmelterStatus,
}

impl Smelter {
    pub fn new(pos: IVec3) -> Smelter {
        let slots = Kind::Smelter.def().slots;
        Smelter {
            pos,
            input: Buffer::new(slots),
            fuel: Buffer::new(slots),
            out: Buffer::new(slots),
            batch: None,
            progress: 0,
            burn: 0,
            outs: Vec::new(),
            next_out: 0,
            status: SmelterStatus::NoOre,
        }
    }

    /// How many of `item` it would take now: fuel or ore, up to that buffer's room.
    pub fn room_for(&self, item: ItemId) -> u32 {
        match sort(item) {
            Some(Sorted::Fuel) => self.fuel.space_for(item),
            Some(Sorted::Ore) => self.input.space_for(item),
            None => 0,
        }
    }

    /// Whether a belt or miner could hand it one `item` now.
    pub fn can_accept(&self, item: ItemId) -> bool {
        self.room_for(item) > 0
    }

    /// Takes one `item` into the fuel or ore buffer; false if it isn't fuel or ore, or doesn't fit.
    pub fn accept(&mut self, item: ItemId) -> bool {
        self.room_for(item) > 0 && self.buffer_for(item).is_some_and(|b| b.add(item, 1) == 0)
    }

    /// The buffer `item` belongs in: fuel, ore, or none.
    pub fn buffer_for(&mut self, item: ItemId) -> Option<&mut Buffer> {
        match sort(item)? {
            Sorted::Fuel => Some(&mut self.fuel),
            Sorted::Ore => Some(&mut self.input),
        }
    }

    /// One tick: work on the batch (starting one if it can), then push an ingot out.
    pub fn step(&mut self, belts: &mut [Belt]) {
        self.work();
        self.out.feed(&self.outs, &mut self.next_out, belts);
    }

    fn work(&mut self) {
        if self.batch.is_none() && !self.start() {
            return;
        }
        let Some(r) = self.recipe() else { return };
        if self.burn == 0 {
            let s = self.fuel.slots[0];
            if let Some(secs) = burn_time(s.item).filter(|_| !s.is_empty()) {
                self.fuel.take(0, 1);
                self.burn = ticks(secs);
            }
        }
        if self.burn == 0 {
            self.status = SmelterStatus::NoFuel;
            return;
        }
        self.status = SmelterStatus::Working;
        self.burn -= 1;
        self.progress += 1;
        if self.progress >= ticks(r.seconds) {
            self.out.add(r.output.0, r.output.1);
            self.batch = None;
            self.progress = 0;
        }
    }

    /// Starts a batch from the ore in `input` if its output fits; sets the status when it can't.
    fn start(&mut self) -> bool {
        let s = self.input.slots[0];
        let Some(i) = machine_recipe_using(SMELTER, s.item).filter(|_| !s.is_empty()) else {
            self.status = SmelterStatus::NoOre;
            return false;
        };
        let r = &MACHINE_RECIPES[i as usize];
        let need = r.inputs.iter().find(|x| x.0 == s.item).map_or(1, |x| x.1);
        if s.count < need {
            self.status = SmelterStatus::NoOre;
            return false;
        }
        if self.out.space_for(r.output.0) < r.output.1 {
            self.status = SmelterStatus::OutputFull;
            return false;
        }
        self.input.take(0, need);
        self.batch = Some(i);
        self.progress = 0;
        true
    }

    fn recipe(&self) -> Option<&'static MachineRecipe> {
        MACHINE_RECIPES.get(self.batch? as usize)
    }

    /// The first readout line, also the panel's status.
    pub fn status_text(&self) -> String {
        match (self.status, self.recipe()) {
            (SmelterStatus::Working, Some(r)) => format!(
                "Smelting {} · {} a minute",
                item::name(r.output.0),
                (r.output.1 as f64 * 60.0 / r.seconds).round() as u32
            ),
            (SmelterStatus::NoFuel, _) => "Out of fuel: bring coal ore or logs".to_string(),
            (SmelterStatus::OutputFull, _) => "Output full: put a belt leading away, or take the ingots".to_string(),
            _ => "Waiting for iron or copper ore".to_string(),
        }
    }

    pub fn panel(&self) -> Panel {
        let total = self.recipe().map_or(1, |r| ticks(r.seconds));
        Panel {
            block: SMELTER,
            recipe: self.batch,
            choosable: false,
            progress: self.progress * 1000 / total,
            fire: self.burn.div_ceil(TICK_RATE),
            slots: vec![
                (ROLE_INPUT, self.input.slots[0]),
                (ROLE_FUEL, self.fuel.slots[0]),
                (ROLE_OUTPUT, self.out.slots[0]),
            ],
            status: self.status_text(),
        }
    }
}

impl Machine for Smelter {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    /// Core state: buffers, batch, fire, round-robin position and status (`outs` comes from `relink`).
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        self.input.write_state(w);
        self.fuel.write_state(w);
        self.out.write_state(w);
        w.bool(self.batch.is_some());
        w.u16(self.batch.unwrap_or(0));
        w.u32(self.progress);
        w.u32(self.burn);
        w.u32(self.next_out as u32);
        w.u8(self.status as u8);
    }

    fn read_state(r: &mut ByteReader) -> Option<Smelter> {
        let mut s = Smelter::new(r.ivec3()?);
        let slots = s.input.slots.len();
        s.input = Buffer::read_state(r, slots)?;
        s.fuel = Buffer::read_state(r, slots)?;
        s.out = Buffer::read_state(r, slots)?;
        let (busy, batch) = (r.bool()?, r.u16()?);
        s.batch = busy.then_some(batch);
        s.progress = r.u32()?;
        s.burn = r.u32()?;
        s.next_out = r.u32()? as usize;
        s.status = *STATUSES.get(r.u8()? as usize)?;
        let valid = s.batch.is_none_or(|b| MACHINE_RECIPES.get(b as usize).is_some_and(|r| r.machine == SMELTER));
        valid.then_some(s)
    }

    /// Its buffers, plus the inputs of an unfinished batch (given back rather than lost).
    fn contents(&self) -> Vec<Stack> {
        let mut all = self.input.contents();
        all.extend(self.fuel.contents());
        all.extend(self.out.contents());
        if let Some(r) = self.recipe() {
            all.extend(r.inputs.iter().map(|&(item, count)| Stack { item, count }));
        }
        all
    }

    /// Status and rate, then what each buffer holds.
    fn describe(&self, _: &Factory) -> String {
        let mut lines = vec![self.status_text()];
        let held = |label: &str, b: &Buffer| {
            let s = b.slots[0];
            (!s.is_empty()).then(|| format!("{label} {} {}", s.count, item::name(s.item)))
        };
        let mut parts: Vec<String> =
            [held("Ore:", &self.input), held("Fuel:", &self.fuel)].into_iter().flatten().collect();
        if self.burn > 0 {
            parts.push(format!("fire for {} s", self.burn.div_ceil(TICK_RATE)));
        }
        if !parts.is_empty() {
            lines.push(parts.join(" · "));
        }
        if let Some(h) = held("Holding", &self.out) {
            lines.push(h);
        }
        lines.push("Right-click to open".to_string());
        lines.join("\n")
    }

    /// A brick furnace with a chimney whose cap is the status lamp.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, _: f64) {
        let body = [tex::SMELTER_TOP, tex::SMELTER_SIDE, tex::SMELTER_TOP];
        push_box(out, rel + Vec3::new(0.0, -0.1, 0.0), 0.0, [0.9, 0.8, 0.9], 0.0, body, false);
        push_box(out, rel + Vec3::new(0.0, 0.36, 0.0), 0.0, [0.28, 0.12, 0.28], 0.0, [tex::FRAME; 3], true);
        let lamp = match self.status {
            SmelterStatus::Working => tex::LAMP_GREEN,
            SmelterStatus::OutputFull => tex::LAMP_YELLOW,
            SmelterStatus::NoFuel => tex::LAMP_RED,
            SmelterStatus::NoOre => tex::FRAME,
        };
        push_box(out, rel + Vec3::new(0.0, 0.46, 0.0), 0.0, [0.3, 0.08, 0.3], 0.0, [lamp; 3], false);
    }
}

enum Sorted {
    Ore,
    Fuel,
}

/// Which buffer an arriving item belongs in, if any.
fn sort(item: ItemId) -> Option<Sorted> {
    if burn_time(item).is_some() {
        Some(Sorted::Fuel)
    } else {
        machine_recipe_using(SMELTER, item).map(|_| Sorted::Ore)
    }
}
