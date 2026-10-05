//! Processors: every machine that turns inputs into outputs (the smelter, the constructor, and the
//! assembler, furnaces and crushers to come) is one `Processor` driven by its spec row (`specs.rs`):
//! which recipe categories it takes, burner or electric, how it picks a recipe, its buffers, its
//! numbers per tier, its model (`model.rs`) and what it says (`view.rs`). Saves from before version 18 held smelters and
//! constructors in lists of their own (`legacy.rs`).
//!
//! Belts, miners and the panel deliver into it; it sorts fuel into its fuel buffer and takes only
//! what a recipe it may make uses (`Pick`; a recipe research still locks counts as unknown), at most a
//! stack of each input when it has a chosen recipe, so one input can't crowd out the others. Like a
//! box, it pushes one item a tick into the next belt leading away. A spec with a footprint
//! (`footprint/`) is several cells big, turned by `dir`, and takes and gives only at its ports.
//!
//! Invariants: a batch uses up its inputs when it starts and only starts when its outputs fit (and,
//! electric, while powered). Work is counted in thousandths of a Mk1 tick: a tier's `speed` times the
//! grid's power share, so batches finish exactly. A burner's fire holds work too, lights a new fuel
//! item only while a batch runs and burns `fuel` thousandths per unit of work. Changing the recipe hands
//! back the inputs, an unfinished batch's included.
//!
//! A recipe's byproducts (its outputs after the first) go to a separate buffer if the spec has one
//! (`side`), which belts take from at the footprint's `Role::Side` ports; once it holds `SIDE_ROOM` it
//! blocks the next batch and the status names the byproduct. Saved only for specs that have one.
//!
//! To add a processor: a spec row (`specs.rs`). New behaviour (flows) goes here.

mod docks;
mod hangar;
mod legacy;
mod model;
mod parts;
mod solar;
mod specs;
mod steam;
mod steam_view;
mod view;
mod work;

use hangar::Hangar;
pub use legacy::{read_constructor, read_smelter};
pub(super) use solar::run as run_renewables;
use solar::Store;
#[cfg(test)]
pub use specs::SPECS;
pub use specs::{makes, spec, Energy, Pick, ProcessSpec, ProcessTier};
use steam::Steam;
pub(super) use steam::{draw_water, link as link_steam, run_turbine};

use crate::block::BlockId;
use crate::bytes::{ByteReader, ByteWriter};
use crate::inventory::Stack;
use crate::item::{stack_size, ItemId};
use crate::math::{IVec3, Vec3};
use crate::recipes::{burn_time, MachineRecipe};

use super::belt::Belt;
use super::buffer::Buffer;
use super::footprint::Role;
use super::{Factory, Machine};

/// Byproducts a machine holds before it stops for want of a belt to take them (a stack is more than
/// a player would notice filling).
const SIDE_ROOM: u32 = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    NoRecipe,
    Working,
    NoInput,
    OutputFull,
    NoPower,
    NoFuel,
    /// A boiler with fuel but no water.
    NoWater,
}

/// Every status, in declaration order: saves store `status as u8`.
const STATUSES: [Status; 7] = [
    Status::NoRecipe,
    Status::Working,
    Status::NoInput,
    Status::OutputFull,
    Status::NoPower,
    Status::NoFuel,
    Status::NoWater,
];

pub struct Processor {
    pub pos: IVec3,
    pub spec: &'static ProcessSpec,
    /// The way its placer faced (`footprint/`); its front faces back the other way.
    pub dir: u8,
    /// Index into `spec.tiers` (0 is Mk1).
    pub tier: u8,
    /// The chosen recipe (`Pick::Chosen`), a `MACHINE_RECIPES` index.
    pub recipe: Option<u16>,
    /// The batch in progress and the work it has had (thousandths of a Mk1 tick).
    pub batch: Option<u16>,
    pub progress: u32,
    /// Work the fire can still fuel (burners, same units).
    pub burn: u32,
    pub input: Buffer,
    pub fuel: Buffer,
    pub out: Buffer,
    /// Byproducts (`spec.side` slots; none for most machines).
    pub side: Buffer,
    /// A boiler's or turbine's steam and water (steam.rs).
    pub steam: Steam,
    /// A panel's or accumulator's charge and flow (solar.rs).
    pub store: Store,
    /// A drone port's fleet out and whether it is busy (hangar.rs; derived).
    pub hangar: Hangar,
    /// Last tick's power share, in thousandths (derived, for the readout).
    pub speed: u32,
    /// Belt indices leading away from its output ports, and from its byproduct ports.
    pub outs: Vec<u32>,
    pub side_outs: Vec<u32>,
    pub next_out: usize,
    pub next_side: usize,
    pub status: Status,
}

impl Processor {
    pub fn new(pos: IVec3, spec: &'static ProcessSpec, tier: u8) -> Processor {
        let [input, fuel, out] = spec.buffers.map(Buffer::new);
        let status = if spec.pick == Pick::Chosen { Status::NoRecipe } else { Status::NoInput };
        let tier = tier.min(spec.tiers.len() as u8 - 1);
        Processor {
            pos,
            spec,
            dir: 0,
            tier,
            recipe: None,
            batch: None,
            progress: 0,
            burn: 0,
            input,
            fuel,
            out,
            side: Buffer::new(spec.side),
            steam: Steam::default(),
            store: Store::default(),
            hangar: Hangar::default(),
            speed: 0,
            outs: Vec::new(),
            side_outs: Vec::new(),
            next_out: 0,
            next_side: 0,
            status,
        }
    }

    /// This tier's numbers.
    pub fn stats(&self) -> &'static ProcessTier {
        &self.spec.tiers[self.tier as usize]
    }

    /// How many of `item` it would take now, up to the room in its buffer. `unlocked`: which machine
    /// recipes research allows, by index.
    pub fn room_for(&self, item: ItemId, unlocked: &[bool]) -> u32 {
        if self.is_fuel(item) {
            return self.fuel.space_for(item);
        }
        if self.spec.pick.stores() {
            return self.out.space_for(item);
        }
        if self.spec.pick == Pick::Hangar {
            return self.hangar_room(item).min(self.input.space_for(item));
        }
        let wanted = match self.spec.pick {
            Pick::Chosen => self.chosen().is_some_and(|r| r.inputs.iter().any(|x| x.0 == item)),
            Pick::ByInput => self.spec.recipe_using(item, unlocked).is_some(),
            Pick::Store | Pick::Hangar | Pick::Load | Pick::Unload => false,
        };
        let cap = if self.spec.pick == Pick::Chosen {
            stack_size(item).saturating_sub(self.input.count(item))
        } else {
            u32::MAX
        };
        if wanted {
            self.input.space_for(item).min(cap)
        } else {
            0
        }
    }

    /// Takes one `item`; false if it doesn't want it or it doesn't fit.
    pub fn accept(&mut self, item: ItemId, unlocked: &[bool]) -> bool {
        self.insert(item, 1, unlocked) == 1
    }

    /// Puts up to `n` of `item` where it belongs; returns how many went in.
    pub fn insert(&mut self, item: ItemId, n: u32, unlocked: &[bool]) -> u32 {
        let put = n.min(self.room_for(item, unlocked));
        let buf = match () {
            _ if self.is_fuel(item) => &mut self.fuel,
            _ if self.spec.pick.stores() => &mut self.out,
            _ => &mut self.input,
        };
        buf.add(item, put);
        put
    }

    /// Switches to `recipe` (one it makes, or `None`) and returns the inputs it held, an unfinished
    /// batch's included.
    pub fn set_recipe(&mut self, recipe: Option<u16>) -> Vec<Stack> {
        let mut back = self.input.contents();
        back.extend(self.batch_inputs());
        self.input = Buffer::new(self.input.slots.len());
        (self.recipe, self.batch, self.progress) = (recipe, None, 0);
        self.status = if recipe.is_some() { Status::NoInput } else { Status::NoRecipe };
        back
    }

    /// One tick at `power` (thousandths: its grid's share, or full for burners): work on the batch
    /// (starting one if it can), then push an item out.
    pub fn step(&mut self, belts: &mut [Belt], power: u32, unlocked: &[bool]) {
        self.speed = power;
        match (self.energy(), self.spec.pick) {
            (Energy::Boiler, _) => self.boil(),
            (Energy::Turbine | Energy::Solar | Energy::Accumulator, _) => {}
            (_, pick) if pick.stores() || pick == Pick::Hangar => {}
            _ => self.work(power, unlocked),
        }
        self.out.feed(&self.outs, &mut self.next_out, belts);
        self.side.feed(&self.side_outs, &mut self.next_side, belts);
    }

    /// Whether it would work this tick if powered (its grid counts it as demand).
    pub fn wants_power(&self, unlocked: &[bool]) -> bool {
        if self.spec.pick == Pick::Hangar {
            return self.hangar.busy;
        }
        self.energy() == Energy::Electric
            && (self.batch.is_some() || self.next(unlocked).is_ok_and(|i| self.blocked(i).is_none()))
    }

    /// Every cell it occupies, its anchor `pos` first.
    pub fn cells(&self) -> Vec<IVec3> {
        self.spec.footprint.cells(self.pos, self.dir)
    }

    /// Whether items arriving into `cell` from the cell `from` beside it go in (through a port).
    pub fn takes_from(&self, cell: IVec3, from: IVec3) -> bool {
        self.spec.footprint.takes(self.pos, self.dir, cell, from)
    }

    /// The faces it gives items out of: a cell and the `DIRS` index it faces.
    pub fn out_faces(&self) -> Vec<(IVec3, u8)> {
        self.spec.footprint.faces(self.pos, self.dir, Role::Out)
    }

    /// The faces it gives byproducts out of.
    pub fn side_faces(&self) -> Vec<(IVec3, u8)> {
        self.spec.footprint.faces(self.pos, self.dir, Role::Side)
    }

    /// How its tier is driven.
    pub fn energy(&self) -> Energy {
        self.stats().energy
    }

    /// kW it draws while it works (0 unless electric).
    pub fn power(&self) -> u32 {
        self.stats().power
    }

    fn is_fuel(&self, item: ItemId) -> bool {
        matches!(self.energy(), Energy::Burner | Energy::Boiler) && burn_time(item).is_some()
    }

    fn chosen(&self) -> Option<&'static MachineRecipe> {
        self.spec.recipe(self.recipe?)
    }

    fn batch_recipe(&self) -> Option<&'static MachineRecipe> {
        self.spec.recipe(self.batch?)
    }

    fn batch_inputs(&self) -> Vec<Stack> {
        let r = self.batch_recipe();
        r.map_or_else(Vec::new, |r| r.inputs.iter().map(|&(item, count)| Stack { item, count }).collect())
    }

    /// `None` if a saved recipe or batch isn't one it makes (damage).
    fn valid(self) -> Option<Processor> {
        let ok = |i: Option<u16>| i.is_none_or(|i| self.spec.recipe(i).is_some());
        (ok(self.recipe) && ok(self.batch)).then_some(self)
    }
}

impl Machine for Processor {
    fn pos(&self) -> IVec3 {
        self.pos
    }

    fn cells(&self) -> Vec<IVec3> {
        Processor::cells(self)
    }

    /// Core state: block, tier and facing, recipe, batch, buffers, work, fire, round-robin position and status,
    /// then the byproduct buffer and its round-robin position if its spec has one (`outs` come from `relink`).
    fn write_state(&self, w: &mut ByteWriter) {
        w.ivec3(self.pos);
        w.u8(self.spec.block);
        w.u8(self.tier);
        w.u8(self.dir);
        for i in [self.recipe, self.batch] {
            w.bool(i.is_some());
            w.u16(i.unwrap_or(0));
        }
        self.input.write_state(w);
        self.fuel.write_state(w);
        self.out.write_state(w);
        w.u32(self.progress);
        w.u32(self.burn);
        w.u32(self.next_out as u32);
        w.u8(self.status as u8);
        if self.spec.side > 0 {
            self.side.write_state(w);
            w.u32(self.next_side as u32);
        }
        if self.energy() == Energy::Boiler {
            w.u32(self.steam.steam);
            w.u32(self.steam.water);
        }
        if self.energy() == Energy::Accumulator {
            w.u32(self.store.charge);
        }
    }

    fn read_state(r: &mut ByteReader) -> Option<Processor> {
        let pos = r.ivec3()?;
        let spec = spec(r.u8()? as BlockId)?;
        let tier = r.u8()?;
        if tier as usize >= spec.tiers.len() {
            return None;
        }
        let mut p = Processor::new(pos, spec, tier);
        // Saves before version 19 had no facing: every processor then was one cell facing north.
        p.dir = if r.version >= 19 { r.u8()? % 4 } else { 0 };
        p.recipe = read_opt(r)?;
        p.batch = read_opt(r)?;
        let [input, fuel, out] = spec.buffers;
        p.input = Buffer::read_state(r, input)?;
        p.fuel = Buffer::read_state(r, fuel)?;
        p.out = Buffer::read_state(r, out)?;
        p.progress = r.u32()?;
        p.burn = r.u32()?;
        p.next_out = r.u32()? as usize;
        p.status = *STATUSES.get(r.u8()? as usize)?;
        if spec.side > 0 {
            p.side = Buffer::read_state(r, spec.side)?;
            p.next_side = r.u32()? as usize;
        }
        if p.energy() == Energy::Boiler {
            p.steam.steam = r.u32()?.min(steam::STEAM_CAP);
            p.steam.water = r.u32()?;
        }
        if p.energy() == Energy::Accumulator {
            p.store.charge = r.u32()?.min(solar::CHARGE_CAP);
        }
        p.valid()
    }

    /// Its buffers, plus the inputs of an unfinished batch (given back rather than lost).
    fn contents(&self) -> Vec<Stack> {
        let mut all = self.input.contents();
        all.extend(self.fuel.contents());
        all.extend(self.out.contents());
        all.extend(self.side.contents());
        all.extend(self.batch_inputs());
        all
    }

    fn describe(&self, f: &Factory) -> String {
        let text = self.readout();
        let source = matches!(self.energy(), Energy::Turbine | Energy::Solar | Energy::Accumulator);
        match steam::grid_line(self, f).filter(|_| source && !f.dirty) {
            Some(grid) => format!("{text}\n{grid}"),
            None => text,
        }
    }

    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64) {
        model::draw(self, out, rel, time);
    }
}

/// An optional recipe index as written: a flag, then the index (0 for none).
fn read_opt(r: &mut ByteReader) -> Option<Option<u16>> {
    let (some, i) = (r.bool()?, r.u16()?);
    Some(some.then_some(i))
}

#[cfg(test)]
mod tests;
