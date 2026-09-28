//! Factory machines: belts (with ramps, lifts and underpasses), miners, storage boxes, processors
//! (smelters, constructors: `process/`), splitters, filters, power (generators, poles), research labs
//! (with the world's `Research`), pipework (`pipes.rs`, `pumping.rs`), quarries and the world's
//! terraforming sites (`sites.rs`). Belts, miners and processors come in tiers (`tiers.rs`).
//!
//! Machines occupy one voxel each (the chunk holds their block id, so collision, targeting and breaking
//! work unchanged) while their state lives here, keyed by position in `at`, and keep running when their
//! chunk is streamed out. `MACHINES` maps a block to its kind and buffer size (several blocks may share
//! a kind). Each kind is a struct in its own file implementing [`Machine`], in its own `Vec`; code acting
//! on one machine matches on its `Slot`. Removal is `swap_remove` plus fixing the moved entry's `at` slot.
//!
//! Core state (DEV_PLAN section 3.4): `update` runs one fixed tick: power (`power.rs`: supply and
//! demand), miners, boxes and processing machines, the powered machines, then belts (downstream first,
//! see `links.rs`), and reports to the view only through `SimEvent`s. Links and the belt order are
//! derived data, rebuilt by `relink` whenever `dirty` is set.
//!
//! A machine that turns inputs into outputs is a processor spec row (`process/specs.rs`), not a kind.
//! To add a machine: its file (struct, `step`, `impl Machine`), a `Kind` and a `Slot` variant with a
//! `MACHINES` row and a `Vec` field (saved in `state.rs`), then follow the compiler through the `match`es
//! (`place`, `remove`, `update`, `links.rs`, `describe.rs`, `render.rs`, `panel.rs`). Its block goes in
//! `block.rs`, its recipe in `recipes.rs`.

mod belt;
mod belt_shape;
mod buffer;
mod describe;
mod generator;
mod lab;
mod links;
mod miner;
mod panel;
mod pipes;
mod power;
mod process;
mod pumping;
mod quarry;
mod render;
mod router;
mod sites;
mod state;
mod storage;
pub mod tiers;
pub mod upgrades;

use rustc_hash::FxHashMap;

use crate::block::{
    BlockId, BELT, CONSTRUCTOR, FACE_BOTTOM, FAST_BELT, FILTER, GENERATOR, LAB, LIFT, MINER, MINER_MK2, OUTLET, PIPE,
    POLE, PUMP, QUARRY, RAMP_DOWN, RAMP_UP, SMELTER, SPLITTER, STORAGE, UNDERPASS_IN, UNDERPASS_OUT,
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
use generator::Generator;
use lab::{step_labs, Lab};
use links::{Sinks, Slot};
use miner::Miner;
use pipes::Pipework;
use power::{Pole, Power};
use process::Processor;
use quarry::Quarry;
use router::Router;
use storage::Storage;

pub use belt::belt_preview;
pub use belt_shape::Shape;
pub use describe::fmt_int;
#[cfg(test)]
pub use miner::MinerStatus;
pub use miner::MINER_TIERS;
pub use panel::{ROLE_FUEL, ROLE_INPUT, ROLE_OUTPUT};
pub use process::{makes, spec as process_spec, Energy};
#[cfg(test)]
pub use process::{ProcessSpec, Status as ProcessStatus, SPECS};
pub use quarry::{survey, DigBox, DEFAULT_DEPTH, DEFAULT_WIDTH, DEPTHS, WIDTHS};
pub use render::{push_box, INSTANCE_FLOATS};
pub use sites::{survey_site, Job, Sites};

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
    Process,
    Router,
    Generator,
    Pole,
    Lab,
    Pipe,
    Quarry,
}

pub struct MachineDef {
    /// The block that is this machine (its name, textures and breaking come from `block.rs`).
    pub block: BlockId,
    pub kind: Kind,
    /// Item stacks per buffer: a box's slots, a miner's output. Belts carry items instead, and
    /// processors take theirs from their spec (0).
    pub slots: usize,
    /// Right-click opens its panel (`panel.rs`) instead of taking what it holds.
    pub panel: bool,
}

/// The machine table: first one row per kind, in `Kind` order (`Kind::def`), then further blocks of
/// an existing kind.
pub const MACHINES: [MachineDef; 21] = [
    MachineDef { block: BELT, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: MINER, kind: Kind::Miner, slots: 1, panel: false },
    MachineDef { block: STORAGE, kind: Kind::Storage, slots: 24, panel: true },
    MachineDef { block: SMELTER, kind: Kind::Process, slots: 0, panel: true },
    MachineDef { block: SPLITTER, kind: Kind::Router, slots: 0, panel: false },
    MachineDef { block: GENERATOR, kind: Kind::Generator, slots: 1, panel: true },
    MachineDef { block: POLE, kind: Kind::Pole, slots: 0, panel: false },
    MachineDef { block: LAB, kind: Kind::Lab, slots: PACKS.len(), panel: true },
    MachineDef { block: PIPE, kind: Kind::Pipe, slots: 0, panel: false },
    MachineDef { block: QUARRY, kind: Kind::Quarry, slots: 4, panel: true },
    MachineDef { block: FILTER, kind: Kind::Router, slots: 0, panel: true },
    MachineDef { block: RAMP_UP, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: RAMP_DOWN, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: LIFT, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: UNDERPASS_IN, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: UNDERPASS_OUT, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: CONSTRUCTOR, kind: Kind::Process, slots: 0, panel: true },
    // Legacy blocks: worlds from before tiers (`tiers.rs`) still hold them; nothing places them now.
    MachineDef { block: MINER_MK2, kind: Kind::Miner, slots: 1, panel: false },
    MachineDef { block: FAST_BELT, kind: Kind::Belt, slots: 0, panel: false },
    MachineDef { block: PUMP, kind: Kind::Pipe, slots: 0, panel: false },
    MachineDef { block: OUTLET, kind: Kind::Pipe, slots: 0, panel: false },
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
    processors: Vec<Processor>,
    routers: Vec<Router>,
    generators: Vec<Generator>,
    poles: Vec<Pole>,
    labs: Vec<Lab>,
    pipework: Vec<Pipework>,
    quarries: Vec<Quarry>,
    /// Grids and last tick's supply and demand (derived, see `power.rs`).
    power: Power,
    at: FxHashMap<IVec3, Slot>,
    /// Belt indices, downstream first.
    order: Vec<u32>,
    dirty: bool,
    pub deposits: Deposits,
    /// The world's research, which labs advance.
    pub research: Research,
    /// The world's terraforming sites (`sites.rs`).
    pub sites: Sites,
    /// Blocks the machines changed this tick, with the block each replaced (pumps and outlets); not
    /// state: `Sim::step` drains it into `block_changed`.
    pub changed: Vec<(IVec3, BlockId)>,
}

impl Factory {
    /// How many machines of `kind` there are.
    pub fn count(&self, kind: Kind) -> usize {
        match kind {
            Kind::Belt => self.belts.len(),
            Kind::Miner => self.miners.len(),
            Kind::Storage => self.storages.len(),
            Kind::Process => self.processors.len(),
            Kind::Router => self.routers.len(),
            Kind::Generator => self.generators.len(),
            Kind::Pole => self.poles.len(),
            Kind::Lab => self.labs.len(),
            Kind::Pipe => self.pipework.len(),
            Kind::Quarry => self.quarries.len(),
        }
    }

    /// How many processors of `block` there are (smelters, constructors).
    pub fn processors_of(&self, block: BlockId) -> usize {
        self.processors.iter().filter(|p| p.spec.block == block).count()
    }

    /// Adds the machine that `block` is at `pos` (nothing for other blocks). `facing` is the placing
    /// player's horizontal direction (belts run that way); `against` is the clicked block, which a
    /// miner drills if it is adjacent. `tier` is the machine's tier for tiered kinds (`tiers.rs`).
    pub fn place(&mut self, world: &mut World, block: BlockId, pos: IVec3, facing: u8, against: IVec3, tier: u8) {
        let Some(def) = machine(block) else { return };
        self.remove(pos);
        let at = &mut self.at;
        match def.kind {
            Kind::Belt => self.add_shaped_belt(pos, facing, Shape::of(block), tier.max((block == FAST_BELT) as u8)),
            Kind::Miner => {
                let drill = face_of(against - pos);
                let deposit = drill.and_then(|_| self.deposits.lookup(world, against));
                self.add_miner(pos, drill.unwrap_or(FACE_BOTTOM as u8), deposit, tier.max((block == MINER_MK2) as u8));
            }
            Kind::Storage => self.add_storage(pos),
            Kind::Process => {
                if let Some(spec) = process::spec(block) {
                    add_to(&mut self.processors, Processor::new(pos, spec, tier), at, Slot::Process);
                }
            }
            Kind::Router => add_to(&mut self.routers, Router::new(pos, facing, block == FILTER), at, Slot::Router),
            Kind::Generator => add_to(&mut self.generators, Generator::new(pos), at, Slot::Generator),
            Kind::Pole => add_to(&mut self.poles, Pole { pos }, at, Slot::Pole),
            Kind::Lab => add_to(&mut self.labs, Lab::new(pos), at, Slot::Lab),
            Kind::Pipe => add_to(&mut self.pipework, Pipework::new(pos, block, facing), at, Slot::Pipe),
            Kind::Quarry => add_to(&mut self.quarries, Quarry::new(pos, facing), at, Slot::Quarry),
        }
        self.dirty = true;
    }

    #[cfg(test)]
    pub fn add_belt(&mut self, pos: IVec3, dir: u8) {
        self.add_shaped_belt(pos, dir, Shape::Flat, 0);
    }

    fn add_shaped_belt(&mut self, pos: IVec3, dir: u8, shape: Shape, tier: u8) {
        self.remove(pos);
        add_to(&mut self.belts, Belt::new(pos, dir, shape, tier), &mut self.at, Slot::Belt);
        self.dirty = true;
    }

    pub fn add_miner(&mut self, pos: IVec3, drill: u8, deposit: Option<DepositKey>, tier: u8) {
        self.remove(pos);
        add_to(&mut self.miners, Miner::new(pos, drill, deposit, tier), &mut self.at, Slot::Miner);
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
            Slot::Process(i) => swap_out(&mut self.processors, i, at, Slot::Process),
            Slot::Router(i) => swap_out(&mut self.routers, i, at, Slot::Router),
            Slot::Generator(i) => swap_out(&mut self.generators, i, at, Slot::Generator),
            Slot::Pole(i) => swap_out(&mut self.poles, i, at, Slot::Pole),
            Slot::Lab(i) => swap_out(&mut self.labs, i, at, Slot::Lab),
            Slot::Pipe(i) => swap_out(&mut self.pipework, i, at, Slot::Pipe),
            Slot::Quarry(i) => swap_out(&mut self.quarries, i, at, Slot::Quarry),
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
            processors,
            routers,
            generators,
            labs,
            pipework,
            quarries,
            power,
            deposits,
            research,
            order,
            changed,
            ..
        } = self;
        let unlocked = research.machine_recipes_unlocked();
        power.balance(generators, miners, processors, routers, labs, pipework, quarries, research, &unlocked);
        let mut sinks = Sinks { storages, processors, routers, generators, labs, unlocked: &unlocked };
        for (m, &p) in miners.iter_mut().zip(&power.miner_pole) {
            m.speed = power.speed(p);
            m.step(deposits, world, tick, belts, &mut sinks, events);
        }
        for (q, &p) in quarries.iter_mut().zip(&power.quarry_pole) {
            q.speed = power.speed(p);
            q.step(world, belts, &mut sinks, changed, events);
        }
        for s in sinks.storages.iter_mut() {
            s.step(belts);
        }
        for (m, &p) in sinks.processors.iter_mut().zip(&power.process_pole) {
            let share = if m.spec.energy == Energy::Burner { power::FULL_SPEED } else { power.speed(p) };
            m.step(belts, share, &unlocked);
        }
        for (r, &p) in sinks.routers.iter_mut().zip(&power.router_pole) {
            r.step(belts, power.speed(p));
        }
        step_labs(sinks.labs, &power.lab_pole, power, research);
        pumping::step_pipework(pipework, &power.pipe_pole, power, world, changed);
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
