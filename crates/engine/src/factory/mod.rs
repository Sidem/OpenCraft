//! Factory machines: conveyor belts, miners and storage boxes.
//!
//! Machines occupy one voxel each (the chunk holds their block id, so collision, targeting and
//! breaking work unchanged) while their state lives here, keyed by position in `at`. Each kind is
//! a `Vec` in its own file (`belt.rs`, `miner.rs`, `storage.rs`); removal is `swap_remove` plus
//! fixing the moved entry's `at` slot. Machines keep running when their chunk is streamed out.
//!
//! Core state (DEV_PLAN section 3.4): `update` runs one fixed tick, miners, then boxes, then belts
//! (downstream first, see `links.rs`), and reports to the view only through `SimEvent`s. Belt links
//! and the belt order are derived data, rebuilt by `relink` whenever `dirty` is set.
//!
//! To add a machine kind (Milestone 2 turns this into a registry, together with the smelter):
//! a file with its struct and `*_step` methods, a `Slot` variant, an `add_*` method, arms in
//! `remove`, `take_contents`, `relink`, `describe` and `write_instances`, and a block id.

mod belt;
mod describe;
mod links;
mod miner;
mod render;
mod storage;

use rustc_hash::FxHashMap;

use crate::block::BlockId;
use crate::bytes::{ByteReader, ByteWriter};
use crate::deposits::{DepositKey, Deposits};
use crate::inventory::{add_to_slots, Stack, MAX_STACK};
use crate::math::IVec3;
use crate::sim::SimEvent;
use crate::world::World;
use crate::TICK;

use belt::{belt_step, Belt};
use miner::Miner;
use storage::Storage;

pub use describe::fmt_int;
#[cfg(test)]
pub use miner::MINER_BUFFER;
pub use miner::{MinerStatus, MINER_RECOVERY};
pub use render::{push_box, INSTANCE_FLOATS};

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

#[derive(Default)]
pub struct Factory {
    belts: Vec<Belt>,
    miners: Vec<Miner>,
    storages: Vec<Storage>,
    at: FxHashMap<IVec3, Slot>,
    /// Belt indices, downstream first.
    order: Vec<u32>,
    dirty: bool,
    pub deposits: Deposits,
}

impl Factory {
    pub fn belt_count(&self) -> usize {
        self.belts.len()
    }

    pub fn miner_count(&self) -> usize {
        self.miners.len()
    }

    pub fn storage_count(&self) -> usize {
        self.storages.len()
    }

    pub fn add_belt(&mut self, pos: IVec3, dir: u8) {
        self.remove(pos);
        self.at.insert(pos, Slot::Belt(self.belts.len() as u32));
        self.belts.push(Belt::new(pos, dir));
        self.dirty = true;
    }

    pub fn add_miner(&mut self, pos: IVec3, drill: u8, deposit: Option<DepositKey>) {
        self.remove(pos);
        self.at.insert(pos, Slot::Miner(self.miners.len() as u32));
        self.miners.push(Miner::new(pos, drill, deposit));
        self.dirty = true;
    }

    pub fn add_storage(&mut self, pos: IVec3) {
        self.remove(pos);
        self.at.insert(pos, Slot::Storage(self.storages.len() as u32));
        self.storages.push(Storage::new(pos));
        self.dirty = true;
    }

    /// Removes the machine at `pos`, returning whatever it was holding or carrying.
    pub fn remove(&mut self, pos: IVec3) -> Vec<Stack> {
        let Some(slot) = self.at.remove(&pos) else { return Vec::new() };
        self.dirty = true;
        match slot {
            Slot::Belt(i) => {
                let b = self.belts.swap_remove(i as usize);
                if let Some(moved) = self.belts.get(i as usize) {
                    self.at.insert(moved.pos, Slot::Belt(i));
                }
                stacks_of(b.items.iter().map(|it| it.item))
            }
            Slot::Miner(i) => {
                let m = self.miners.swap_remove(i as usize);
                if let Some(moved) = self.miners.get(i as usize) {
                    self.at.insert(moved.pos, Slot::Miner(i));
                }
                if m.held > 0 {
                    vec![Stack { item: m.ore, count: m.held }]
                } else {
                    Vec::new()
                }
            }
            Slot::Storage(i) => {
                let s = self.storages.swap_remove(i as usize);
                if let Some(moved) = self.storages.get(i as usize) {
                    self.at.insert(moved.pos, Slot::Storage(i));
                }
                s.slots.into_iter().filter(|s| !s.is_empty()).collect()
            }
        }
    }

    /// Right-click on a box or miner: moves its contents into `take(item, count)`, which returns
    /// how many it accepted. Returns false if there is no such machine at `pos`.
    pub fn take_contents(&mut self, pos: IVec3, mut take: impl FnMut(BlockId, u32) -> u32) -> bool {
        match self.at.get(&pos) {
            Some(Slot::Storage(i)) => {
                for s in self.storages[*i as usize].slots.iter_mut().filter(|s| !s.is_empty()) {
                    s.count -= take(s.item, s.count).min(s.count);
                    if s.count == 0 {
                        *s = Stack::default();
                    }
                }
                true
            }
            Some(Slot::Miner(i)) => {
                let m = &mut self.miners[*i as usize];
                if m.held > 0 {
                    m.held -= take(m.ore, m.held).min(m.held);
                }
                true
            }
            _ => false,
        }
    }

    /// Core state: every machine in `Vec` order, then the deposits. `at`, `order` and the links are
    /// derived from these.
    pub fn write_state(&self, w: &mut ByteWriter) {
        w.count(self.belts.len());
        self.belts.iter().for_each(|b| b.write_state(w));
        w.count(self.miners.len());
        self.miners.iter().for_each(|m| m.write_state(w));
        w.count(self.storages.len());
        self.storages.iter().for_each(|s| s.write_state(w));
        self.deposits.write_state(w);
    }

    /// Reads what `write_state` wrote; `world` must already hold the saved edits (deposits survey it).
    /// Links are rebuilt at the first `update`. Two machines in one place is damage.
    pub fn read_state(world: &mut World, r: &mut ByteReader) -> Option<Factory> {
        let mut f = Factory { dirty: true, ..Factory::default() };
        for _ in 0..r.count()? {
            let b = Belt::read_state(r)?;
            if f.at.insert(b.pos, Slot::Belt(f.belts.len() as u32)).is_some() {
                return None;
            }
            f.belts.push(b);
        }
        for _ in 0..r.count()? {
            let m = Miner::read_state(r)?;
            if f.at.insert(m.pos, Slot::Miner(f.miners.len() as u32)).is_some() {
                return None;
            }
            f.miners.push(m);
        }
        for _ in 0..r.count()? {
            let s = Storage::read_state(r)?;
            if f.at.insert(s.pos, Slot::Storage(f.storages.len() as u32)).is_some() {
                return None;
            }
            f.storages.push(s);
        }
        f.deposits.read_state(world, r)?;
        Some(f)
    }

    /// Runs every machine for one tick (`TICK` seconds). `tick` must differ between calls: the
    /// deposits' shared draw budgets refill once per tick.
    pub fn update(&mut self, world: &mut World, tick: u64, events: &mut Vec<SimEvent>) {
        if self.dirty {
            self.relink();
        }
        let dt = TICK;
        let Factory { belts, miners, storages, deposits, order, .. } = self;
        for m in miners.iter_mut() {
            let drawn = m.draw_step(deposits, world, tick, dt);
            m.output_step(belts, storages);
            m.pulse_step(drawn, events);
        }
        for s in storages.iter_mut() {
            s.output_step(belts);
        }
        belt_step(belts, storages, order, dt);
    }

    #[cfg(test)]
    pub fn storage_count_at(&self, pos: IVec3, item: BlockId) -> u32 {
        match self.at.get(&pos) {
            Some(Slot::Storage(i)) => {
                self.storages[*i as usize].slots.iter().filter(|s| s.item == item).map(|s| s.count).sum()
            }
            _ => 0,
        }
    }

    #[cfg(test)]
    fn belt_at(&self, pos: IVec3) -> &Belt {
        match self.at.get(&pos) {
            Some(Slot::Belt(i)) => &self.belts[*i as usize],
            _ => panic!("no belt at {pos:?}"),
        }
    }

    #[cfg(test)]
    pub fn miner_at(&self, pos: IVec3) -> &Miner {
        match self.at.get(&pos) {
            Some(Slot::Miner(i)) => &self.miners[*i as usize],
            _ => panic!("no miner at {pos:?}"),
        }
    }
}

/// Where a belt, miner or box delivers to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Link {
    None,
    /// `mid`: joining from the side, so the item enters halfway along the target belt.
    Belt {
        belt: u32,
        mid: bool,
    },
    Storage(u32),
}

/// What occupies a position: an index into the matching machine `Vec`.
#[derive(Clone, Copy, Debug)]
enum Slot {
    Belt(u32),
    Miner(u32),
    Storage(u32),
}

#[inline]
fn opposite(dir: u8) -> u8 {
    (dir + 2) % 4
}

/// Hands one item to a link. `overflow` is how far past the end of the source belt it already is.
fn deliver(belts: &mut [Belt], storages: &mut [Storage], link: Link, item: BlockId, overflow: f32) -> bool {
    match link {
        Link::None => false,
        Link::Belt { belt, mid } => belts[belt as usize].accept(item, mid, overflow),
        Link::Storage(s) => add_to_slots(&mut storages[s as usize].slots, item, 1) == 0,
    }
}

/// Merges loose items into stacks (for dropping a machine's contents).
fn stacks_of(items: impl Iterator<Item = BlockId>) -> Vec<Stack> {
    let mut out: Vec<Stack> = Vec::new();
    for item in items {
        match out.iter_mut().find(|s| s.item == item && s.count < MAX_STACK) {
            Some(s) => s.count += 1,
            None => out.push(Stack { item, count: 1 }),
        }
    }
    out
}

#[cfg(test)]
mod tests;
