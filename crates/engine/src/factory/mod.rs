//! Factory machines: conveyor belts (with ramps, lifts and underpasses), miners, storage boxes, smelters, constructors,
//! splitters, filters, power (generators, poles) and research labs (with the world's `Research`).
//!
//! Machines occupy one voxel each (the chunk holds their block id, so collision, targeting and
//! breaking work unchanged) while their state lives here, keyed by position in `at`. The machine
//! table `MACHINES` maps a block to its kind and buffer size (several blocks may share a kind). Each kind is a struct in its own file
//! implementing [`Machine`] (state bytes, contents, readout, model), stored in its own `Vec`; code
//! that acts on one machine finds it through one `match` on `Slot`. Removal is `swap_remove` plus
//! fixing the moved entry's `at` slot. Machines keep running when their chunk is streamed out.
//!
//! Core state (DEV_PLAN section 3.4): `update` runs one fixed tick: power (`power.rs`: this tick's
//! supply and demand), miners, boxes and processing machines, the powered machines, then belts (downstream first, see `links.rs`),
//! and reports to the view only
//! through `SimEvent`s. Links and the belt order are derived data, rebuilt by `relink` whenever `dirty` is set.
//!
//! To add a machine: its file (struct, `step`, `impl Machine`), a `Kind` and a `Slot` variant with a
//! `MACHINES` row and a `Vec` field (saved in `state.rs`), then follow the compiler through the `match`es
//! (`place`, `remove`, `update`, `links.rs`, `describe.rs`, `render.rs`, `panel.rs`). Its block goes in `block.rs`, its
//! recipe in `recipes.rs`.

mod belt;
mod belt_shape;
mod buffer;
mod constructor;
mod describe;
mod generator;
mod lab;
mod links;
mod miner;
mod panel;
mod power;
mod render;
mod router;
mod smelter;
mod state;
mod storage;

use rustc_hash::FxHashMap;

use crate::block::{
    BlockId, BELT, CONSTRUCTOR, FACE_BOTTOM, FAST_BELT, FILTER, GENERATOR, LAB, LIFT, MINER, MINER_MK2, POLE,
    RAMP_DOWN, RAMP_UP, SMELTER, SPLITTER, STORAGE, UNDERPASS_IN, UNDERPASS_OUT,
};
use crate::bytes::{ByteReader, ByteWriter};
use crate::deposits::{DepositKey, Deposits};
use crate::inventory::Stack;
#[cfg(test)]
use crate::item::ItemId;
use crate::math::{IVec3, Vec3};
use crate::research::{Research, PACKS};
use crate::sim::SimEvent;
use crate::world::World;
use crate::{TICK, TICK_RATE};

use belt::{belt_step, Belt};
use belt_shape::Shape;
use constructor::Constructor;
use generator::Generator;
use lab::{step_labs, Lab};
use links::{Sinks, Slot};
use miner::Miner;
use power::{Pole, Power};
use router::Router;
use smelter::Smelter;
use storage::Storage;

#[cfg(test)]
pub use constructor::ConstructorStatus;
pub use describe::fmt_int;
#[cfg(test)]
pub use miner::MinerStatus;
pub use miner::{MINER_RECOVERY, MK2_RECOVERY};
pub use panel::{ROLE_FUEL, ROLE_INPUT, ROLE_OUTPUT};
pub use render::{push_box, INSTANCE_FLOATS};
#[cfg(test)]
pub use smelter::SmelterStatus;

/// Horizontal directions in player-yaw quarter turns: 0 = -Z (north), 1 = +X, 2 = +Z, 3 = -X.
pub const DIRS: [IVec3; 4] = [IVec3::new(0, 0, -1), IVec3::new(1, 0, 0), IVec3::new(0, 0, 1), IVec3::new(-1, 0, 0)];
/// Block face normals in mesher order: +X, -X, +Y, -Y, +Z, -Z.
pub const FACES: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

/// The horizontal direction a player facing `yaw` looks along.
pub fn dir_from_yaw(yaw: f64) -> u8 {
    ((yaw / std::f64::consts::FRAC_PI_2).round() as i32).rem_euclid(4) as u8
}

pub fn face_of(v: IVec3) -> Option<u8> {
    FACES.iter().position(|&f| f == v).map(|i| i as u8)
}

/// Machine kinds, in `MACHINES` order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Belt,
    Miner,
    Storage,
    Smelter,
    Constructor,
    Router,
    Generator,
    Pole,
    Lab,
}

pub struct MachineDef {
    /// The block that is this machine (its name, textures and breaking come from `block.rs`).
    pub block: BlockId,
    pub kind: Kind,
    /// Item stacks per buffer: a box's slots, a miner's output, each of a smelter's ore, fuel and
    /// output buffers. Belts carry items instead (0).
    pub slots: usize,
    /// Right-click opens its panel (`panel.rs`) instead of taking what it holds.
    pub panel: bool,
}

/// The machine table: first one row per kind, in `Kind` order (`Kind::def`), then further blocks of
/// an existing kind.
pub const MACHINES: [MachineDef; 17] = [
    MachineDef { block: BELT, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: MINER, kind: Kind::Miner, slots: 1, panel: false },
    MachineDef { block: STORAGE, kind: Kind::Storage, slots: 24, panel: true },
    MachineDef { block: SMELTER, kind: Kind::Smelter, slots: 1, panel: true },
    MachineDef { block: CONSTRUCTOR, kind: Kind::Constructor, slots: 1, panel: true },
    MachineDef { block: SPLITTER, kind: Kind::Router, slots: 0, panel: false },
    MachineDef { block: GENERATOR, kind: Kind::Generator, slots: 1, panel: true },
    MachineDef { block: POLE, kind: Kind::Pole, slots: 0, panel: false },
    MachineDef { block: LAB, kind: Kind::Lab, slots: PACKS.len(), panel: true },
    MachineDef { block: FILTER, kind: Kind::Router, slots: 0, panel: true },
    MachineDef { block: RAMP_UP, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: RAMP_DOWN, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: LIFT, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: UNDERPASS_IN, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: UNDERPASS_OUT, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: MINER_MK2, kind: Kind::Miner, slots: 1, panel: false },
    MachineDef { block: FAST_BELT, kind: Kind::Belt, slots: 0, panel: false },
];

impl Kind {
    pub fn def(self) -> &'static MachineDef {
        &MACHINES[self as usize]
    }
}

/// The machine `block` is, if it is one.
pub fn machine(block: BlockId) -> Option<&'static MachineDef> {
    MACHINES.iter().find(|m| m.block == block)
}

/// What every machine kind provides. Static dispatch only: callers `match` on `Slot` or loop over
/// one kind's `Vec`.
trait Machine: Sized {
    fn pos(&self) -> IVec3;
    /// Its core state (derived data such as links is left out).
    fn write_state(&self, w: &mut ByteWriter);
    fn read_state(r: &mut ByteReader) -> Option<Self>;
    /// Everything it holds or carries, dropped when it is removed.
    fn contents(&self) -> Vec<Stack>;
    /// Readout lines for the HUD ("" for nothing to say).
    fn describe(&self, f: &Factory) -> String;
    /// Box instances for its model; `rel` is its cell centre relative to the camera.
    fn model(&self, out: &mut Vec<f32>, rel: Vec3, time: f64);
}

#[derive(Default)]
pub struct Factory {
    belts: Vec<Belt>,
    miners: Vec<Miner>,
    storages: Vec<Storage>,
    smelters: Vec<Smelter>,
    constructors: Vec<Constructor>,
    routers: Vec<Router>,
    generators: Vec<Generator>,
    poles: Vec<Pole>,
    labs: Vec<Lab>,
    /// Grids and last tick's supply and demand (derived, see `power.rs`).
    power: Power,
    at: FxHashMap<IVec3, Slot>,
    /// Belt indices, downstream first.
    order: Vec<u32>,
    dirty: bool,
    pub deposits: Deposits,
    /// The world's research, which labs advance.
    pub research: Research,
}

impl Factory {
    /// How many machines of `kind` there are.
    pub fn count(&self, kind: Kind) -> usize {
        match kind {
            Kind::Belt => self.belts.len(),
            Kind::Miner => self.miners.len(),
            Kind::Storage => self.storages.len(),
            Kind::Smelter => self.smelters.len(),
            Kind::Constructor => self.constructors.len(),
            Kind::Router => self.routers.len(),
            Kind::Generator => self.generators.len(),
            Kind::Pole => self.poles.len(),
            Kind::Lab => self.labs.len(),
        }
    }

    /// Adds the machine that `block` is at `pos` (nothing for other blocks). `facing` is the placing
    /// player's horizontal direction (belts run that way); `against` is the clicked block, which a
    /// miner drills if it is adjacent.
    pub fn place(&mut self, world: &mut World, block: BlockId, pos: IVec3, facing: u8, against: IVec3) {
        let Some(def) = machine(block) else { return };
        match def.kind {
            Kind::Belt => self.add_shaped_belt(pos, facing, Shape::of(block), block == FAST_BELT),
            Kind::Miner => {
                let drill = face_of(against - pos);
                let deposit = drill.and_then(|_| self.deposits.lookup(world, against));
                self.add_miner(pos, drill.unwrap_or(FACE_BOTTOM as u8), deposit, block == MINER_MK2);
            }
            Kind::Storage => self.add_storage(pos),
            Kind::Smelter => {
                self.remove(pos);
                add_to(&mut self.smelters, Smelter::new(pos), &mut self.at, Slot::Smelter);
            }
            Kind::Constructor => {
                self.remove(pos);
                add_to(&mut self.constructors, Constructor::new(pos), &mut self.at, Slot::Constructor);
            }
            Kind::Router => {
                self.remove(pos);
                add_to(&mut self.routers, Router::new(pos, facing, block == FILTER), &mut self.at, Slot::Router);
            }
            Kind::Generator => {
                self.remove(pos);
                add_to(&mut self.generators, Generator::new(pos), &mut self.at, Slot::Generator);
            }
            Kind::Pole => {
                self.remove(pos);
                add_to(&mut self.poles, Pole { pos }, &mut self.at, Slot::Pole);
            }
            Kind::Lab => {
                self.remove(pos);
                add_to(&mut self.labs, Lab::new(pos), &mut self.at, Slot::Lab);
            }
        }
        self.dirty = true;
    }

    #[cfg(test)]
    pub fn add_belt(&mut self, pos: IVec3, dir: u8) {
        self.add_shaped_belt(pos, dir, Shape::Flat, false);
    }

    fn add_shaped_belt(&mut self, pos: IVec3, dir: u8, shape: Shape, fast: bool) {
        self.remove(pos);
        add_to(&mut self.belts, Belt::new(pos, dir, shape, fast), &mut self.at, Slot::Belt);
        self.dirty = true;
    }

    pub fn add_miner(&mut self, pos: IVec3, drill: u8, deposit: Option<DepositKey>, mk2: bool) {
        self.remove(pos);
        add_to(&mut self.miners, Miner::new(pos, drill, deposit, mk2), &mut self.at, Slot::Miner);
        self.dirty = true;
    }

    pub fn add_storage(&mut self, pos: IVec3) {
        self.remove(pos);
        add_to(&mut self.storages, Storage::new(pos), &mut self.at, Slot::Storage);
        self.dirty = true;
    }

    /// Removes the machine at `pos`, returning whatever it was holding or carrying.
    pub fn remove(&mut self, pos: IVec3) -> Vec<Stack> {
        let Some(slot) = self.at.remove(&pos) else { return Vec::new() };
        self.dirty = true;
        let at = &mut self.at;
        match slot {
            Slot::Belt(i) => swap_out(&mut self.belts, i, at, Slot::Belt),
            Slot::Miner(i) => swap_out(&mut self.miners, i, at, Slot::Miner),
            Slot::Storage(i) => swap_out(&mut self.storages, i, at, Slot::Storage),
            Slot::Smelter(i) => swap_out(&mut self.smelters, i, at, Slot::Smelter),
            Slot::Constructor(i) => swap_out(&mut self.constructors, i, at, Slot::Constructor),
            Slot::Router(i) => swap_out(&mut self.routers, i, at, Slot::Router),
            Slot::Generator(i) => swap_out(&mut self.generators, i, at, Slot::Generator),
            Slot::Pole(i) => swap_out(&mut self.poles, i, at, Slot::Pole),
            Slot::Lab(i) => swap_out(&mut self.labs, i, at, Slot::Lab),
        }
    }

    /// Runs every machine for one tick (`TICK` seconds). `tick` must differ between calls: the
    /// deposits' shared draw budgets refill once per tick.
    pub fn update(&mut self, world: &mut World, tick: u64, events: &mut Vec<SimEvent>) {
        if self.dirty {
            self.relink();
        }
        let Factory {
            belts,
            miners,
            storages,
            smelters,
            constructors,
            routers,
            generators,
            labs,
            power,
            deposits,
            research,
            order,
            ..
        } = self;
        power.balance(generators, miners, constructors, routers, labs, research);
        let mut sinks = Sinks { storages, smelters, constructors, routers, generators, labs };
        for (m, &p) in miners.iter_mut().zip(&power.miner_pole) {
            m.speed = power.speed(p);
            m.step(deposits, world, tick, belts, &mut sinks, events);
        }
        for s in sinks.storages.iter_mut() {
            s.step(belts);
        }
        for s in sinks.smelters.iter_mut() {
            s.step(belts);
        }
        for (c, &p) in sinks.constructors.iter_mut().zip(&power.constructor_pole) {
            c.step(belts, power.speed(p));
        }
        for (r, &p) in sinks.routers.iter_mut().zip(&power.router_pole) {
            r.step(belts, power.speed(p));
        }
        step_labs(sinks.labs, &power.lab_pole, power, research);
        belt_step(belts, &mut sinks, order, TICK);
    }
}

#[inline]
fn opposite(dir: u8) -> u8 {
    (dir + 2) % 4
}

/// Whole ticks in `seconds` (machine work is counted in ticks).
fn ticks(seconds: f64) -> u32 {
    (seconds * TICK_RATE as f64).round() as u32
}

/// Appends `m` to its kind's list and indexes its position.
fn add_to<T: Machine>(list: &mut Vec<T>, m: T, at: &mut FxHashMap<IVec3, Slot>, slot: fn(u32) -> Slot) {
    at.insert(m.pos(), slot(list.len() as u32));
    list.push(m);
}

/// Removes entry `i` (already gone from `at`), re-indexes the entry moved into its place, and
/// returns the removed machine's contents.
fn swap_out<T: Machine>(
    list: &mut Vec<T>,
    i: u32,
    at: &mut FxHashMap<IVec3, Slot>,
    slot: fn(u32) -> Slot,
) -> Vec<Stack> {
    let m = list.swap_remove(i as usize);
    if let Some(moved) = list.get(i as usize) {
        at.insert(moved.pos(), slot(i));
    }
    m.contents()
}

#[cfg(test)]
mod tests;
